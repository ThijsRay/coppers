# Coppers

Coppers is a test harness for Rust with the intention of determining the evolution of power consumptions of a program between different versions. In order to determine the power consumption of a program coppers replaces the existing testing harness in Rust and runs each test written by the developer of a Rust program to approximate the power consumption of a program.

## Requirements
This test harness targets the Rust nightly toolchain because it relies on unstable features of the Rust compiler.
* First, make sure that you have installed the nightly toolchain with `rustup install nightly`
* Then, enable the nightly toolchain on this repository with `rustup override set nightly`

## Usage 
To enable the custom test runner in your project, add this to your `Cargo.toml` file.
```toml
[dev-dependencies]
coppers = { git = "https://github.com/ThijsRay/coppers" }
```
Add the following two lines at the top of your crate root (most likely `lib.rs` or `main.rs`)
```rust
#![feature(custom_test_frameworks)]
#![test_runner(coppers::runner)]
```

## How to run Coppers

During development Coppers can be build with a regular `cargo build`. Since the program needs root access to the sensors (as long as the rights to the sensors is not changed), the program needs to be run with super user rights (e.g. `sudo ./target/debug/coppers`).

### Common problems

**RAPL sensors are not enabled**

You can enable on the RAPL sensors with modprobe in the following way `modprobe intel_rapl_common` for Linux kernels of >= 5. Do you have a kernel version of < 5, then use `modprobe intel_rapl`
