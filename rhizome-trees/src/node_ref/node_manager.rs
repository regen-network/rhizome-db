use std::ops::Deref;
use std::sync::{Arc, RwLock};
use anyhow::{anyhow, Result};
use lru::LruCache;
use crate::node_ref::r#impl::{Node, NodeHandle, NodeRef};
use crate::node_ref::r#impl::{NodeRefInner};
use crate::node_ref::node_store::{NodeStore};

/// The NodeManager functions as an abstraction over node storage and caching
/// which properly handles reading and writing of nodes.
pub struct NodeManager<N: Node> {
    /// The underlying node store.
    pub node_store: Box<dyn NodeStore<N>>,

    /// The in-memory cache of nodes read from storage.
    pub cache: Arc<LruCache<N::Ptr, N>>,
}

impl<N: Node> Clone for NodeManager<N> {
    fn clone(&self) -> Self {
        Self{ node_store: self.node_store.clone_ref(), cache: self.cache.clone() }
    }
}

impl<N: Node> NodeManager<N> {
    /// Reads the node from memory, the cache or storage.
    pub fn read<'a>(&self, node_ref: &'a NodeRef<N>) -> Result<Option<NodeHandle<'a, N>>> {
        match node_ref {
            NodeRef::Inner(inner) => self.read_inner(inner),
            NodeRef::Empty => Ok(None),
        }
    }

    fn read_inner<'a>(&self, inner: &'a Arc<RwLock<NodeRefInner<N>>>) -> Result<Option<NodeHandle<'a, N>>> {
        let mut cache_copy: Option<NodeRefInner<N>> = None;
        let res = match inner.read() {
            Ok(node_ref) => {
                let mut have_mem_node = false;
                if let NodeRefInner::MemNode(_) = node_ref.deref() {
                    have_mem_node = true;
                }
                if have_mem_node {
                    return Ok(Some(NodeHandle::Mem(node_ref)))
                }
                match node_ref.deref() {
                    NodeRefInner::MemNode(_) => Err(anyhow!("unexpected case")),
                    NodeRefInner::StoredNode { pointer: disk_pointer, cached } => {
                        if let Some(node) = cached.upgrade() {
                            Ok(Some(NodeHandle::Arc(node)))
                        } else {
                            let node = self.node_store.read(disk_pointer)?;
                            cache_copy = Some(NodeRefInner::StoredNode {
                                pointer: disk_pointer.clone(),
                                cached: Arc::downgrade(&node),
                            });
                            Ok(Some(NodeHandle::Arc(node)))
                        }
                    }
                }
            }
            Err(_) => {
                Err(anyhow!("poison error"))
            }
        }?;

        // try to cache the weak pointer
        if let Some(new_ref) = cache_copy {
            if let Ok(mut node_ref) = inner.try_write() {
                *node_ref = new_ref;
            }
        }
        Ok(res)
    }


    /// Takes the node if editable is true, the node is only in memory and the reference count to it
    /// is 1. Otherwise clones the node. This method can be used to build trees that can be used
    /// either as persistent or transient data structures. A transient version of a persistent data
    /// structure is one where nodes that are not shared with any other version are safe to mutate.
    pub fn take_or_clone(&self, node_ref: NodeRef<N>, editable: bool) -> Result<Option<(N, bool)>> {
        if !editable {
            return match self.read(&node_ref)? {
                None => Ok(None),
                Some(node_arc) => Ok(Some(((*node_arc).clone(), false)))
            };
        }

        match node_ref {
            NodeRef::Inner(inner) => {
                match Arc::try_unwrap(inner) {
                    Ok(inner) => {
                        match RwLock::into_inner(inner) {
                            Ok(inner) => {
                                match inner {
                                    NodeRefInner::MemNode(node) => {
                                        Ok(Some((node, true)))
                                    }
                                    NodeRefInner::StoredNode { pointer: disk_pointer, cached } => {
                                        if let Some(node_arc) = cached.upgrade() {
                                            return Ok(Some(((*node_arc).clone(), false)));
                                        }

                                        let node_arc = self.node_store.read(&disk_pointer)?;
                                        Ok(Some(((*node_arc).clone(), false)))
                                    }
                                }
                            }
                            Err(err) => Err(anyhow!("poison error: {:?}", err))
                        }
                    }
                    Err(inner_arc) => {
                        if let Some(node_arc) = self.read_inner(&inner_arc)? {
                            Ok(Some(((*node_arc).clone(), false)))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            NodeRef::Empty => Ok(None)
        }
    }

    /// Saves the node to the underlying storage (if it has not been saved already)
    /// and returns the pointer to it which can be used by the parent node
    /// pointing to this node in its serialized form to point to its child.
    /// If the node has already been saved, its reference count will be incremented
    /// whenever save is called.
    pub fn save(&mut self, node_ref: &NodeRef<N>) -> Result<Option<N::Ptr>> {
        match node_ref {
            NodeRef::Inner(inner) => {
                match inner.write() {
                    Ok(mut node_ref) => {
                        match node_ref.deref() {
                            NodeRefInner::MemNode(node) => {
                                // TODO cache
                                let ptr = self.node_store.insert(node)?;
                                *node_ref = NodeRefInner::StoredNode {
                                    pointer: ptr.clone(),
                                    cached: Default::default(), // TODO weak pointer from cached Arc
                                };
                                Ok(Some(ptr))
                            }
                            NodeRefInner::StoredNode { pointer: disk_pointer, .. } => {
                                let _ = self.node_store.inc_ref_count(disk_pointer)?;
                                Ok(Some(disk_pointer.clone()))
                            }
                        }
                    }
                    Err(_) => {
                        Err(anyhow!("poison error"))
                    }
                }
            }
            NodeRef::Empty => {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use coverage_helper::test;
    use lru::LruCache;
    use crate::node_ref::node_store::MemNodeStore;
    use crate::node_ref::NodeManager;
    use crate::node_ref::r#impl::TestNode;

    #[test]
    fn test_manager() {
        let mut mgr = NodeManager{
            node_store: Box::new(MemNodeStore::<TestNode<usize>>::default()),
            cache: Arc::new(LruCache::unbounded()),
        };
    }
}