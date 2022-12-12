//! Primitives structures for referencing tree nodes which allow
//! for nodes to be serialized a variety of different storage backends
//! and easy creation of transient data structures.

mod r#impl;
mod node_store;
mod node_manager;

pub use crate::node_ref::r#impl::{Node, NodeHandle, NodeRef};
pub use crate::node_ref::node_store::{NodeStore, NullNodeStore};
pub use crate::node_ref::node_manager::NodeManager;

