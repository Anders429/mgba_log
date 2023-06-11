# Changelog

## Unreleased
### Changed
- Marked `init()` as `unsafe`.
### Fixed
- Synchronization bug when using the `fatal!` macro.

## 0.2.0 - 2023-06-10
### Changed
- Removed `voladdress` as a dependency.
### Fixed
- Synchronization issues when logging from interrupt handler while already logging in main execution.

## 0.1.0 - 2023-06-06
### Added
- `init()` function to initialize logging.
- `Error` enum to represent potential errors.
- `fatal!` macro for logging at the `Fatal` level.
