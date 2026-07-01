use toml_const::from_toml;

#[from_toml("tests/basic.toml")]
mod config{
    #[root]
    pub struct Config {
        pub sample_rate: u32,
    }
}

#[test]
fn single_int(){
    assert_eq!(config::CONFIG.sample_rate, 48000)
}
