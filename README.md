# Concrete-Config

`concrete-config` is an [attribute-macro](https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro_attribute-attribute) that allows you to define a `const` constructable struct containing any build-time constants you want,
and point to a TOML file that should contain the values that should be put into those build-time constants.

`concrete-config` then reads your config struct, reads your TOML file, and constructs a `const` instance of your struct containing the values from the TOML file.
Along the way, 
