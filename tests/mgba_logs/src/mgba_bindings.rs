#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
)]

// Bindings are generated by build.rs.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
