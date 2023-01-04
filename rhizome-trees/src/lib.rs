//! A library of serializable, persistent data structures with optional
//! merkle hashes.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(coverage_nightly, feature(no_coverage))]

extern crate core;

pub mod tree;
pub mod node_ref;

#[cfg(test)] #[macro_use]
extern crate assert_matches;
