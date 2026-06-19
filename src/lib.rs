#![no_std]
#![cfg_attr(test, no_main)]
#![allow(unused_features)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(const_convert)]
#![feature(const_index)]
#![feature(const_result_trait_fn)]
#![feature(const_trait_impl)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod io;
