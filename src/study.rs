//! Study notes whose code blocks are compiled and run as doctests, so every
//! snippet in `docs/` is guaranteed to be current. Run them with
//! `cargo test --doc`.

#[doc = include_str!("../docs/vec_and_hashmap.md")]
pub mod vec_and_hashmap {}

#[doc = include_str!("../docs/input_output.md")]
pub mod input_output {}
