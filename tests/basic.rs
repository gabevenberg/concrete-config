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

#[test]
fn single_int() {
    assert_eq!(single_int::CONFIG.sample_rate, 48000)
}

#[test]
fn static_str(){
    assert_eq!(static_str::CONFIG.ssid, "test")
}
