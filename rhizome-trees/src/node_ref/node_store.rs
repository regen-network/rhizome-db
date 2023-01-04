use std::collections::HashMap;
use anyhow::{anyhow, Result};
use std::sync::{Arc, RwLock};

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
    fn delete(&mut self, ptr: &N::Ptr) -> Result<()>;

    /// Increments the reference count of the node in the storage medium
    /// and returns the current reference count.
    fn inc_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;

    /// Decrements the reference count of the node in the storage medium
    /// and returns the reference count.
    fn dec_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;

    /// Clones the node store dynamically.
    fn clone_ref(&self) -> Box<dyn NodeStore<N>>;
}

/// An node store backed by memory for testing.
#[derive(Default, Clone)]
pub(crate) struct MemNodeStore<N: Node<Ptr=usize>>(MemNodeStoreInner<N>);

pub(crate) type MemNodeStoreInner<N> = Arc<RwLock<HashMap<usize, (u32, Arc<N>)>>>;

impl<N: Node<Ptr=usize> + 'static> NodeStore<N> for MemNodeStore<N>
{
    fn insert(&mut self, node: &N) -> Result<N::Ptr> {
        match self.0.write() {
            Ok(mut hm) => {
                let n = hm.len() + 1;
                hm.insert(n, (1, Arc::new(node.clone())));
                Ok(n)
            }
            Err(_) => Err(anyhow!("poison"))
        }
    }

    fn read(&self, ptr: &N::Ptr) -> Result<Arc<N>> {
        match self.0.read() {
            Ok(hm) => {
                match hm.get(ptr) {
                    None => Err(anyhow!("not found")),
                    Some((_, n)) => Ok(n.clone())
                }
            }
            Err(_) => Err(anyhow!("poison"))
        }
    }

    fn delete(&mut self, ptr: &N::Ptr) -> Result<()> {
        match self.0.write() {
            Ok(mut hm) => {
                hm.remove(ptr);
                Ok(())
            }
            Err(_) => Err(anyhow!("poison"))
        }
    }

    fn inc_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32> {
        match self.0.write() {
            Ok(mut hm) => {
                match hm.get_mut(ptr) {
                    None => Err(anyhow!("not found")),
                    Some((rc, _)) => {
                        *rc += 1;
                        Ok(*rc)
                    }
                }
            }
            Err(_) => Err(anyhow!("poison"))
        }
    }

    fn dec_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32> {
        match self.0.write() {
            Ok(mut hm) => {
                match hm.get_mut(ptr) {
                    None => Err(anyhow!("not found")),
                    Some((rc, _)) => {
                        if *rc == 0 {
                            return Ok(0)
                        }
                        *rc -= 1;
                        Ok(*rc)
                    }
                }
            }
            Err(_) => Err(anyhow!("poison"))
        }
    }

    fn clone_ref(&self) -> Box<dyn NodeStore<N>> {
        Box::new(self.clone())
    }
}
