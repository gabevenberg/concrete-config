use concrete_config::from_toml;

#[from_toml("tests/single_int.toml")]
mod single_int {
    #[root]
    pub struct Config {
        pub sample_rate: u32,
    }
}

#[test]
fn single_int() {
    assert_eq!(single_int::CONFIG.sample_rate, 48000)
}

#[from_toml("tests/static_str.toml")]
mod static_str {
    #[root]
    pub struct Config {
        pub ssid: &'static str,
    }
}

#[test]
fn static_str() {
    assert_eq!(static_str::CONFIG.ssid, "test")
}

#[from_toml("tests/array.toml")]
mod array {
    #[root]
    pub struct Config {
        pub awnser_bytes: [u8; 2],
    }
}

#[test]
fn array() {
    assert_eq!(array::CONFIG.awnser_bytes, [42, 64])
}

#[from_toml("tests/enums.toml")]
mod r#enum {

    #[derive(Debug, Eq, PartialEq)]
    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }

    #[root]
    pub struct Config {
        pub log_level: LogLevel,
    }
}

#[test]
fn r#enum() {
    assert_eq!(r#enum::CONFIG.log_level, r#enum::LogLevel::Warn)
}

#[from_toml("tests/full.toml")]
mod full {
    #[root]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Config {
        pub version: u32,
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

    #[derive(Debug, Eq, PartialEq)]
    pub struct Led {
        pub pin: u8,
        pub pattern: [u8; 3],
    }
}

#[test]
fn full() {
    assert_eq!(
        full::CONFIG,
        full::Config {
            version: 3,
            uart: full::Uart {
                baud: 115200,
                stop_bits: 1,
                parity: full::Parity::Even,
                data_bits: 8,
            },
            leds: [
                full::Led {
                    pin: 10,
                    pattern: [64, 255, 32],
                },
                full::Led {
                    pin: 12,
                    pattern: [255, 128, 16],
                },
            ]
        }
    )
}
