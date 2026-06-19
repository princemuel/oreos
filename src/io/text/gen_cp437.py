#!/usr/bin/env python3
"""
Generates `CP437_TABLE` (decode array) and `encode()` (match-based encoder)
Rust source from the Unicode Consortium's CP437.TXT mapping file.

Source: https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP437.TXT
Format: three tab-separated columns per line:
    0xXX    0xXXXX    #UNICODE CHARACTER NAME

Run:
    python3 gen_cp437.py CP437.TXT > cp437_generated.rs

This is a one-time codegen step. re-run by hand if CP437.TXT changes.
The output is meant to be reviewed and pasted in, not blindly trusted.
"""

import sys


def parse(path: str) -> dict[int, int]:
    """Returns {cp437_byte: unicode_codepoint}, one entry per line."""
    table: dict[int, int] = {}
    with open(path, encoding="utf-8") as f:
        for lineno, raw in enumerate(f, start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) < 2:
                print(f"warning: skipping malformed line {lineno}: {line!r}", file=sys.stderr)
                continue
            byte = int(parts[0], 16)
            codepoint = int(parts[1], 16)
            if byte in table:
                print(f"warning: duplicate byte 0x{byte:02x} at line {lineno}", file=sys.stderr)
            table[byte] = codepoint

    if len(table) != 256:
        print(f"warning: expected 256 entries, got {len(table)}", file=sys.stderr)

    missing = [b for b in range(256) if b not in table]
    if missing:
        print(f"warning: missing bytes: {[hex(b) for b in missing]}", file=sys.stderr)

    return table


def rust_char_literal(codepoint: int) -> str:
    """Renders a Rust char literal, escaping control/quote/backslash chars."""
    ch = chr(codepoint)
    if ch == "\\":
        return "'\\\\'"
    if ch == "'":
        return "'\\''"
    if ch == "\0":
        return "'\\0'"
    if ch == "\n":
        return "'\\n'"
    if ch == "\r":
        return "'\\r'"
    if ch == "\t":
        return "'\\t'"
    if codepoint < 0x20 or codepoint == 0x7F:
        return f"'\\u{{{codepoint:x}}}'"
    return f"'{ch}'"


def emit_table(table: dict[int, int]) -> str:
    lines = ["/// Maps each CP437 byte (0x00..=0xFF) to its `char`, generated from"]
    lines.append("/// the Unicode Consortium's CP437.TXT mapping table:")
    lines.append("/// <https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP437.TXT>")
    lines.append("///")
    lines.append("/// Do not hand-edit; regenerate with `gen_cp437.py` instead.")
    lines.append("#[rustfmt::skip]")
    lines.append("const CP437_TABLE: [char; 256] = [")
    row: list[str] = []
    for byte in range(256):
        row.append(rust_char_literal(table[byte]))
        if len(row) == 8:
            lines.append("    " + ", ".join(row) + ",")
            row = []
    if row:
        lines.append("    " + ", ".join(row) + ",")
    lines.append("];")
    return "\n".join(lines)


def emit_encode(table: dict[int, int]) -> str:
    """
    Emits a const fn encode(char) -> Option<u8> as a match statement,
    grouping multiple codepoints->same-byte where they coincide (none do
    in the raw 1:1 CP437.TXT mapping, but kept generic in case you merge
    in aliases by hand later).
    """
    lines = ["/// Encodes `c` to its CP437 byte. Generated 1:1 from [`CP437_TABLE`];"]
    lines.append("/// see that array's doc comment for source/regeneration info.")
    lines.append("#[must_use]")
    lines.append("#[expect(clippy::too_many_lines)]")
    lines.append("pub const fn encode(c: char) -> Option<u8> {")
    lines.append("    Some(match c {")
    for byte in range(256):
        cp = table[byte]
        ch_lit = rust_char_literal(cp)
        lines.append(f"        {ch_lit} => 0x{byte:02x},")
    lines.append("        _ => return None,")
    lines.append("    })")
    lines.append("}")
    return "\n".join(lines)

def emit_decode() -> str:
    lines = ["/// Decodes a CP437 byte to its `char`."]
    lines.append("#[must_use]")
    lines.append("#[expect(clippy::indexing_slicing)]")
    lines.append("pub const fn decode(byte: u8) -> char { CP437_TABLE[usize::from(byte)] }")
    return "\n".join(lines)



def main() -> None:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} CP437.TXT", file=sys.stderr)
        sys.exit(1)

    table = parse(sys.argv[1])

    print("// AUTO-GENERATED — DO NOT EDIT BY HAND.")
    print("// Regenerate with: python3 gen_cp437.py CP437.txt > cp437_generated.rs")
    print()
    print(emit_table(table))
    print()
    print(emit_encode(table))
    print()
    print(emit_decode())



if __name__ == "__main__":
    main()
