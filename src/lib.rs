//! This is the entry point of the library. Everything is exported from here.

#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results)]

extern crate rand;
extern crate sdl2;

pub mod constants;
pub mod dijkstra_map;
pub mod grid;
pub mod language;
pub mod mobiles;
pub mod statics;
pub mod templates;
pub mod types;
pub mod ui;
pub mod utils;
