#![allow(clippy::assertions_on_constants)]
use concrete_config::concrete_toml;

#[concrete_toml("tests/single_int.toml")]
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

#[concrete_toml("tests/static_str.toml")]
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

#[concrete_toml("tests/bool.toml")]
mod bool {
    #[root]
    pub struct Config {
        pub enabled: bool,
        pub verbose: bool,
    }
}

#[test]
fn bool() {
    assert!(bool::CONFIG.enabled);
    assert!(!bool::CONFIG.verbose);
}

#[concrete_toml("tests/array.toml")]
mod array {
    #[root]
    pub struct Config {
        pub answer_bytes: [u8; 2],
    }
}

#[test]
fn array() {
    assert_eq!(array::CONFIG.answer_bytes, [42, 64])
}

#[concrete_toml("tests/slice.toml")]
mod slice {
    #[root]
    pub struct Config {
        pub answer_bytes: &'static [u8],
    }
}

#[test]
fn slice() {
    assert_eq!(slice::CONFIG.answer_bytes, [42, 64])
}

#[concrete_toml("tests/tuple.toml")]
mod tuple {
    #[root]
    pub struct Config {
        pub sensor: (u8, &'static str),
        pub device_id: (u16,),
    }
}

#[test]
fn tuple() {
    assert_eq!(tuple::CONFIG.sensor, (4, "bme280"));
    assert_eq!(tuple::CONFIG.device_id, (1001,));
}

#[concrete_toml("tests/floats.toml")]
mod floats {
    #[root]
    pub struct Config {
        pub gain: f32,
        pub big: f64,
        pub positive_inf: f32,
        pub negative_inf: f64,
        pub not_a_number: f32,
    }
}

#[test]
fn floats() {
    assert_eq!(floats::CONFIG.gain, 2.75);
    assert_eq!(floats::CONFIG.big, 1.5e300);
    assert!(
        floats::CONFIG.positive_inf.is_infinite() && floats::CONFIG.positive_inf.is_sign_positive()
    );
    assert!(
        floats::CONFIG.negative_inf.is_infinite() && floats::CONFIG.negative_inf.is_sign_negative()
    );
    assert!(floats::CONFIG.not_a_number.is_nan());
}

#[concrete_toml("tests/enums.toml")]
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

#[concrete_toml("tests/full.toml")]
mod full {
    #[root]
    #[derive(Debug, PartialEq)]
    pub struct Config {
        pub version: u32,
        pub debug: bool,
        pub sensor: (u8, &'static str),
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

#[test]
fn full() {
    assert_eq!(
        full::CONFIG,
        full::Config {
            version: 3,
            debug: true,
            sensor: (4, "bme280"),
            uart: full::Uart {
                baud: 115200,
                stop_bits: 1,
                parity: full::Parity::Even,
                data_bits: 8,
            },
            leds: [
                full::Led {
                    pin: 10,
                    pattern: &[64, 255, 32],
                    pattern_time: 0.5,
                },
                full::Led {
                    pin: 12,
                    pattern: &[255, 128, 16],
                    pattern_time: 1.25,
                },
            ]
        }
    )
}
