# Changelog

## Unreleased

## v0.6.0

### Added

- Add basic tests
- Add `TransmitError` and check error in `write_byte`
- Implement embedded_hal serial traits

### Changed

- Use register names instead of arrays
- Derive `Copy` and `Eq` for bare enums
-

### Removed

- Remove toggle methods

### Security

- The `uart` module is private
- Mark some methods as unsafe
  - `from_base_address`
  - `set_base_address`
- Mark some methods as private
  - methods that directly access the whole register

## v0.5.0

- Add several bitflags of status registers
- Add `#[inline]` to most read/write methods
- **BREAKING CHANGE** Traits from `embedded_hal` are removed due to `is_interrupt_pending` will reset THREI

## v0.4.2

- Don't enable transmitter_holding_register_empty_interrupt in `init()`
  - It seems that 16550 used in qemu keeps triggering THREI when THR is empty

## v0.4.1

- fix: IIR[0] == 0 when interrupt is pending

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
