//! A library of serializable, persistent data structures with optional
//! merkle hashes.

#![deny(missing_docs)]
#![deny(rustdoc::missing_doc_code_examples)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(coverage_nightly, feature(no_coverage))]

pub mod tree;
pub mod node_ref;
