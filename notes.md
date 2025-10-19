# Notes

Cargo supports different target systems through the --target parameter.

The target is described by a so-called target triple, which describes *the CPU architecture, the vendor, the operating system, and the ABI*.
For example, the x86_64-unknown-linux-gnu target triple describes a system with an x86_64 CPU, no clear vendor, and a Linux operating system with the GNU ABI. Rust supports many different target triples, including arm-linux-androideabi for Android or wasm32-unknown-unknown for WebAssembly.

For our target system, however, we require some special configuration parameters (e.g. no underlying OS), so none of the existing target triples fits. Fortunately, Rust allows us to define our own target through a JSON file. For example, a JSON file that describes the x86_64-unknown-linux-gnu target looks like this:

```json
{
  "llvm-target": "x86_64-unknown-linux-gnu",
  "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "target-endian": "little",
  "target-pointer-width": 64,
  "target-c-int-width": 32,
  "os": "linux",
  "executables": true,
  "linker-flavor": "gcc",
  "pre-link-args": ["-m64"],
  "morestack": false
}
```

the CPU architecture, the vendor, the operating system, and the ABI.
