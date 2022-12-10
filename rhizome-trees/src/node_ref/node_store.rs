use anyhow::{anyhow, Result};
use std::sync::{Arc};

use crate::node_ref::r#impl::{Node};

/// The trait to be implied by all node storage mediums.
pub trait NodeStore<N : Node> {
    /// Inserts a new node into storage and returns the pointer to it.
    /// Nodes are always initialized with a reference count of one.
    fn insert(&mut self, node: &N) -> Result<N::Ptr>;

    /// Reads a node from storage by its pointer.
    fn read(&self, ptr: &N::Ptr) -> Result<Arc<N>>;

    /// Deletes the node with the provided pointer from storage.
    /// It is expected that the node manager has safely ensured
    /// that the reference count of this node has been decremented
    /// to zero before calling delete.
    fn delete(&self, ptr: &N::Ptr) -> Result<()>;

    /// Increments the reference count of the node in the storage medium
    /// and returns the current reference count.
    fn inc_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;

    /// Decrements the reference count of the node in the storage medium
    /// and returns the reference count.
    fn dec_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;

    /// Clones the node store dynamically.
    fn clone(&self) -> Box<dyn NodeStore<N>>;
}

/// An empty node store that neither saves nor deletes nodes.
pub struct NullNodeStore {}

impl<N: Node> NodeStore<N> for NullNodeStore
{
    fn insert(&mut self, _node: &N) -> Result<N::Ptr> {
        Err(anyhow!("not implemented"))
    }

    fn read(&self, _ptr: &N::Ptr) -> Result<Arc<N>> {
        Err(anyhow!("not implemented"))
    }

    fn delete(&self, _ptr: &N::Ptr) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn inc_ref_count(&mut self, _ptr: &N::Ptr) -> Result<u32> {
        Ok(1)
    }

    fn dec_ref_count(&mut self, _ptr: &N::Ptr) -> Result<u32> {
        Ok(1)
    }

    fn clone(&self) -> Box<dyn NodeStore<N>> {
        Box::new(Self{})
    }
}
