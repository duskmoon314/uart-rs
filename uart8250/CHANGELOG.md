# Changelog

## v0.4.0

- **BREAKING CHANGE** `MmioUart8250` is restructured to be more rusty
  - use `reg: &'a mut Registers` instead of `base_address: usize` to access register via `self.reg.rw[0]`
- **BREAKING CHANGE** `init()` enable transmitter_holding_register_empty_interrupt now

## v0.3.2

- Add `enable_*` `disable_*` methods to provide more specific usage
- Add `set_base_address` to change `base_address`

## v0.3.1

- Remove incorrect `write_fmt`

## v0.3.0

- **BREAKING CHANGE** `MmioUart8250` is restructured
  - From unit struct to classic C struct
  - Change associated functions to methods to allow runtime setup

## v0.2.0

- **BREAKING CHANGE** `ChipFIFOInfo` rename to `ChipFifoInfo`
- Add `embedded` and `fmt` features that impl traits

## v0.1.0

- Basic function of `MmioUart8250`
