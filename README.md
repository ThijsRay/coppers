# Coppers

Coppers is a test harnass for Rust with the intention of determining the evolution of power consumptions of a program between different versions. In order to determine the power consumption of a program coppers replaces the existing testing harness in Rust and runs each test written by the developer of a Rust program to approximate the power consumption of a program.

## How to run Coppers

During development Coppers can be build with a regular `cargo build`. Since the program needs root access to the sensors (as long as the rights to the sensors is not changed), the program needs to be run with super user rights (e.g. `sudo ./target/debug/coppers`).

### Common problems

**RAPL sensors are not enabled**

You can enable on the RAPL sensors with modprobe in the following way `modprobe intel_rapl_common` for Linux kernels of >= 5. Do you have a kernel version of < 5, then use `modprobe intel_rapl`
