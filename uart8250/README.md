# uart8250

This crate provide a unit struct with many methods to operate uart 8250.

[REF: Serial Programming/8250 UART Programming](https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#UART_Registers)

**Noticed:** This crate may have problems. Any help would be welcomed, even if your help will bring about **breaking change**. Please feel free to start an Issue or a PR.

Currently I **cannot guarantee** the stability of this crate, and it is likely to introduce destructive updates (including but not limited to renaming of structs, renaming of functions and methods, code restructuring). So fixing the dependency version should be a good way to go.

Besides, this crate currently is not following [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). Please feel free to start an Issue or a PR to help me fix this.

## Usage

```rust
type Uart = MmioUart8250<0x1000_0100>;
Uart::init(11_059_200, 115200);
if let Some(c) = Uart::read_byte() {
    //...
}
```
