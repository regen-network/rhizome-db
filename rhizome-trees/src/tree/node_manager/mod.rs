pub mod node_ref;
pub mod node_store;

use std::ops::Deref;
use std::sync::{Arc, RwLock};
use anyhow::{anyhow, Result};
use lru::LruCache;
use crate::tree::node_manager::node_ref::{Node, NodeRef, NodeRefInner};
use crate::tree::node_manager::node_store::{NodeStore, NullNodeStore};

pub struct NodeManager<N: Node> {
    node_store: Arc<dyn NodeStore<N>>,
    cache: LruCache<N::Ptr, N>,
}

impl <N: Node> Default for NodeManager<N> {
    fn default() -> Self {
        NodeManager{
            node_store: Arc::new(NullNodeStore{}),
            cache: LruCache::unbounded(),
        }
    }
}

impl<N: Node> NodeManager<N> {
    pub fn read(&self, node_ref: &NodeRef<N>) -> Result<Option<Arc<N>>> {
        match node_ref {
            NodeRef::Inner(inner) => self.read_inner(inner),
            NodeRef::Empty => Ok(None),
        }
    }

    fn read_inner(&self, inner: &Arc<RwLock<NodeRefInner<N>>>) -> Result<Option<Arc<N>>> {
        let mut cache_copy: Option<NodeRefInner<N>> = None;
        let res = match inner.read() {
            Ok(node_ref) => {
                match node_ref.deref() {
                    NodeRefInner::MemNode(node) => Ok(Some(node.clone())),
                    NodeRefInner::DiskNode { disk_pointer, cached } => {
                        if let Some(node) = cached.upgrade() {
                            Ok(Some(node))
                        } else {
                            let node = self.node_store.read(disk_pointer)?;
                            cache_copy = Some(NodeRefInner::DiskNode {
                                disk_pointer: disk_pointer.clone(),
                                cached: Arc::downgrade(&node),
                            });
                            Ok(Some(node))
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
                                    NodeRefInner::MemNode(node_arc) => {
                                        match Arc::try_unwrap(node_arc) {
                                            Ok(node) => Ok(Some((node, true))),
                                            Err(node_arc) => Ok(Some(((*node_arc).clone(), false)))
                                        }
                                    }
                                    NodeRefInner::DiskNode { disk_pointer, cached } => {
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

    pub fn save(&mut self, node_ref: &NodeRef<N>) -> Result<Option<N::Ptr>> {
        match node_ref {
            NodeRef::Inner(inner) => {
                match inner.write() {
                    Ok(mut node_ref) => {
                        match node_ref.deref() {
                            NodeRefInner::MemNode(node) => {
                                let ptr = self.node_store.create(node)?;
                                *node_ref = NodeRefInner::DiskNode {
                                    disk_pointer: ptr.clone(),
                                    cached: Arc::downgrade(node),
                                };
                                Ok(Some(ptr))
                            }
                            NodeRefInner::DiskNode { disk_pointer, .. } => {
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
