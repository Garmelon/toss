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
mod coords;
mod frame;
mod styled;
mod terminal;
mod widget;
pub mod widgets;
mod widthdb;
mod wrap;

pub use coords::*;
pub use frame::*;
pub use styled::*;
pub use terminal::*;
pub use widget::*;
pub use widthdb::*;
