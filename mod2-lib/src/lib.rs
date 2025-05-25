#![feature(ptr_as_ref_unchecked)]
#![feature(ptr_metadata)]
#![feature(vec_into_raw_parts)]
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(rustdoc::private_intra_doc_links)]

pub mod api;
pub mod core;

// Global Configuration
/// Indentation amount for displayed structures.
pub(crate) const DISPLAY_INDENT: usize = 2;

/// The type to use for symbol, term, and DAG node structural hashes.
pub type HashType = u32;

// ToDo: Do UNDEFINED the right way. Is this great? No. But it's convenient.
/// Sentinel Value
const UNDEFINED: i32 = -1;
/// Sentinel Value
const NONE     : i32 = -1;

