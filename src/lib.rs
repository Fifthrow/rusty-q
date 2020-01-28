//#![feature(lang_items)]
// #![feature(lang_items, box_patterns)]
#[macro_use]
extern crate lazy_static;
extern crate bitflags;
extern crate libc;
extern crate nix;
extern crate num;

include!(concat!(env!("OUT_DIR"), "/symbols.rs"));

pub mod k;
pub mod kapi;
pub mod kbindings;
pub mod types;

#[cfg(feature = "api")]
pub mod api;
