#![forbid(unsafe_code)]
// Rustc lint groups
#![warn(future_incompatible)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
// Rustc lints
#![warn(noop_method_call)]
#![warn(single_use_lifetimes)]
// Clippy lints
#![warn(clippy::use_self)]

mod buffer;
pub mod frame;
pub mod styled;
pub mod terminal;
pub mod widthdb;
mod wrap;
