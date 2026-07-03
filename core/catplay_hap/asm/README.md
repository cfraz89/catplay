# asm

This directory contains assembly source files for various architectures, adapted from OpenSSL and other sources, to provide highly optimized ChaCha20 implementations for use in the `fast_chacha` crate.

## Structure

- Each file targets a specific architecture, operating system, or ABI (e.g., `chacha-x86_64-linux.S`, `chacha-armv8-linux.S`, `chacha-mips-o32.S`, etc.).
- Header files (e.g., `arm_arch.h`, `mips_arch.h`) provide platform-specific macros and definitions.
- `.NOTES` files contain licensing, and author information.

## Usage

Only a subset of these files are currently used and integrated into the build process, depending on platform support and project needs.  
The build script (`build.rs`) selects the appropriate assembly file for your target at compile time.

## Contributing

**Not all files in this directory are used by the project at the moment.**  
This is intentional: these files are provided as a foundation and an opportunity for contributors to:

- Add support for new platforms or operating systems
- Improve or update existing assembly implementations
- Refactor, document, or clean up unused files

If you are interested in contributing, feel free to open issues or pull requests to expand or improve the assembly support in this crate!

## Licensing

See [LICENSE.openssl](LICENSE.openssl) and [LICENSE.cryptogams](LICENSE.cryptogams) and `.NOTES` files for licensing information regarding the assembly code.