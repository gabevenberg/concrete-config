use toml_const::from_toml;

#[from_toml("tests/single_int.toml")]
mod single_int {
    #[root]
    pub struct Config {
        pub sample_rate: u32,
    }
}

#[from_toml("tests/static_str.toml")]
mod static_str {
    #[root]
    pub struct Config {
        pub ssid: &'static str,
    }
}

#[from_toml("tests/array.toml")]
mod array {
    #[root]
    pub struct Config {
        pub awnser_bytes: [u8; 2],
    }
}

#[from_toml("tests/enums.toml")]
mod r#enum {
    #[allow(unused)]

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
fn single_int() {
    assert_eq!(single_int::CONFIG.sample_rate, 48000)
}

#[test]
fn static_str() {
    assert_eq!(static_str::CONFIG.ssid, "test")
}

#[test]
fn array() {
    assert_eq!(array::CONFIG.awnser_bytes, [42, 64])
}

#[test]
fn r#enum() {
    assert_eq!(r#enum::CONFIG.log_level, r#enum::LogLevel::Warn)
}
