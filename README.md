# Concrete-Config

`concrete-config` is an [attribute-macro](https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro_attribute-attribute) for baking build-time constants into your binary.
You define a `const`-constructable struct, and point the macro at a TOML file that fills in the values.
As the generated code is just type definitions and a `const` declaration, it works perfectly in embedded `no_std` environments; in fact, that's exactly what it was designed for.

`concrete-config` then reads your config struct, reads your TOML file, and constructs a `const` instance of your struct containing the values from the TOML file.
Along the way, `concrete-config` checks that the TOML file matches your struct definitions, making sure that all TOML fields map to a struct field, and that all struct fields have a value.

It currently supports user-defined structs and unit enums, all sizes of integers and floats, booleans, `&'static str`, `&'static [T]` slices, and fixed size arrays of any other supported type.

## Usage

First off, add `concrete-config` to your dependencies with `cargo add concrete-config`.

The `#[concrete_toml()]` proc-macro takes the filename of a TOML file as its singular argument (relative to `Cargo.toml`), and must be attached to a module.
The module contains all type definitions needed to construct your root struct.

Some things to keep in mind:
* TOML keys must match field names exactly. `concrete-config` does not perform any case normalization.
* Same for enums, the TOML string must match the capitalization of the enum variant exactly.
* You must mark exactly one struct as `#[root]`. This is the struct that corresponds to the root table of the TOML document.
* Any `#[derive()]`s or other attributes not owned by `concrete-config` will be passed through.
* The resulting `const` declaration will be named `CONFIG`. This is currently hardcoded, but if there is need I could make it another argument to the proc-macro.

Perhaps it is easiest to show an example:

If you have the following in your rust code:
```rust
use concrete_config::concrete_toml;

#[concrete_toml("tests/full.toml")]
mod config {
    #[root]
    #[derive(Debug, PartialEq)]
    pub struct Config {
        pub version: u32,
        pub debug: bool,
        pub uart: Uart,
        pub leds: [Led; 2],
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Uart {
        pub baud: u32,
        pub stop_bits: u8,
        pub parity: Parity,
        pub data_bits: u8,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Parity {
        No,
        Even,
        Odd,
    }

    #[derive(Debug, PartialEq)]
    pub struct Led {
        pub pin: u8,
        pub pattern: &'static [u8],
        pub pattern_time: f32,
    }
}

assert_eq!(config::CONFIG.version, 3);
assert!(config::CONFIG.debug);
assert_eq!(config::CONFIG.uart.parity, config::Parity::Even);
assert_eq!(config::CONFIG.leds[1].pattern, &[255, 128, 16]);
assert_eq!(config::CONFIG.leds[0].pattern_time, 0.5);
```

And the following content in `tests/full.toml`:

```toml
version = 3
debug = true

[uart]
baud = 115200
stop_bits = 1
parity = "Even"
data_bits = 8

[[leds]]
pin = 10
pattern = [64, 255, 32]
pattern_time = 0.5

[[leds]]
pin = 12
pattern = [255, 128, 16]
pattern_time = 1.25

```

Then the `concrete_toml` macro will expand the config module to this (run through rustfmt, of course):

```rust
mod config {
    #![allow(dead_code)]

    #[derive(Debug, PartialEq)]
    pub struct Config {
        pub version: u32,
        pub debug: bool,
        pub uart: Uart,
        pub leds: [Led; 2],
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Uart {
        pub baud: u32,
        pub stop_bits: u8,
        pub parity: Parity,
        pub data_bits: u8,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Parity {
        No,
        Even,
        Odd,
    }

    #[derive(Debug, PartialEq)]
    pub struct Led {
        pub pin: u8,
        pub pattern: &'static [u8],
        pub pattern_time: f32,
    }

    pub const CONFIG: Config = Config {
        version: 3u32,
        debug: true,
        uart: Uart {
            baud: 115200u32,
            stop_bits: 1u8,
            parity: Parity::Even,
            data_bits: 8u8,
        },
        leds: [
            Led {
                pin: 10u8,
                pattern: &[64u8, 255u8, 32u8],
                pattern_time: 0.5f32,
            },
            Led {
                pin: 12u8,
                pattern: &[255u8, 128u8, 16u8],
                pattern_time: 1.25f32,
            },
        ],
    };
    const _: usize = include_bytes!("tests/full.toml").len();
}
```

Then, for example, in `main()`, you can use:
```rust ignore
set_uart_baud(config::CONFIG.uart.baud);
```

The macro passes through the struct definitions, and adds a `pub const CONFIG` `const` literal that contains the values from the TOML file.
If there is any field in the TOML file that is not in the struct,
or any field in the struct that is not in the TOML file, the macro will output a well-formatted compilation error.

Some notes on the expansion:
* The `#![allow(dead_code)]` is automatically added to the module, as it is very likely if you include enums in your config that in a given compilation not all of them will be constructed.
  This is to be expected, but cargo does not know that and marks it as dead code.
* The `const _: usize = include_bytes!("tests/full.toml").len()` is a way of telling cargo that this file depends on `tests/full.toml`, and to rebuild if it changes.
  If no optimizations happen, a single usize is included in the binary; however any dead code elimination will remove even that.
  Under no circumstances is the complete contents of the config file included in the binary.
* As macros run on the host, bounds checking for `usize` and `isize` will be for the host architecture, not the target architecture.
  Be careful when using these types.
  `rustc` does perform a bounds check on literals for the target architecture, but the error message is not pretty.

## Limitations/not supported yet
The following are not supported and will produce compiler errors:

* Future features:
    * Tuple support
    * Data carrying enums
    * Tuple structs
    * Option support for fields that may or may not be in the TOML file
    * Attribute for default values
    * custom `const` declaration name
* Maybe, only if demand is there:
    * Serialization formats other than TOML
    * Specialized parsing for specific types. Think `base64` encoded strings to a `&'static [u8]`, or `IpAddr` types.
* Will not ever be supported:
    * Types that are not const-constructable.
    * Arbitrary `serde` types, macros cannot run code from the token stream they consume, so we cannot use arbitrary serde deserializers.

## Minimum Supported Rust Version

Due to the use of let-chains, the MSRV for this crate is rust 1.88. If there is a need for a lower MSRV, I'd happily accept a PR removing those let-chains.

## Licence

This crate is licenced under the [EUPL-1.2](https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12), which is a copyleft licence compatible with many other copyleft licences.
If for whatever reason you absolutely cannot include copyleft code for your project, contact me privately and we may be able to work out an individual licence.
