use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak};
use anyhow::{anyhow, Result};
use lru::LruCache;

pub trait Node: Sync + Send + Clone {
    type Ptr: Sync + Send + Clone + std::fmt::Debug;
}

#[derive(Debug, Clone)]
pub enum NodeRef<N: Node>
{
    Inner(Arc<RwLock<NodeRefInner<N>>>),
    Empty,
}

#[derive(Debug, Clone)]
pub enum NodeRefInner<N: Node> {
    MemNode(Arc<N>),
    DiskNode { disk_pointer: N::Ptr, cached: Weak<N> },
}

pub struct NodeManager<N: Node> {
    node_store: Arc<dyn NodeStore<N>>,
    cache: LruCache<N::Ptr, N>,
}

pub trait NodeStore<N : Node> {
    fn create(&mut self, node: &N) -> Result<N::Ptr>;
    fn read(&self, ptr: &N::Ptr) -> Result<Arc<N>>;
    fn try_update(&mut self, ptr: &N::Ptr, node: &N) -> Result<Option<N::Ptr>>;
    fn delete(&mut self, ptr: &N::Ptr) -> Result<()>;
    fn inc_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;
    fn dec_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;
}

// #[derive(Debug, Clone)]
// pub enum ValueRef<Value, Ptr> {
//     MemValue()
// }

impl <N: Node> NodeRef<N> {
    pub fn new(node: N) -> Self {
        NodeRef::Inner(Arc::new(RwLock::new(NodeRefInner::MemNode(Arc::new(node)))))
    }

    pub fn read(&self, node_store: &dyn NodeStore<N>) -> Result<Option<Arc<N>>> {
        match self {
            NodeRef::Inner(inner) => NodeRef::read_inner(inner, node_store),
            NodeRef::Empty => Ok(None),
        }
    }

    fn read_inner(inner: &Arc<RwLock<NodeRefInner<N>>>, node_store: &dyn NodeStore<N>) -> Result<Option<Arc<N>>> {
        let mut cache_copy: Option<NodeRefInner<N>> = None;
        let res = match inner.read() {
            Ok(node_ref) => {
                match node_ref.deref() {
                    NodeRefInner::MemNode(node) => Ok(Some(node.clone())),
                    NodeRefInner::DiskNode { disk_pointer, cached } => {
                        if let Some(node) = cached.upgrade() {
                            Ok(Some(node))
                        } else {
                            let node = node_store.read(disk_pointer)?;
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


    // pub fn read_or_clone(self, node_store: &dyn NodeStore<Node, Ptr>) -> Result<Option<Node>> {
    //     match self {
    //         NodeRef::Inner(inner) => {
    //             match Arc::try_unwrap(inner) {
    //                 Ok(inner) => {
    //                     let inner = RwLock::into_inner(inner)?;
    //                     match inner {
    //                         NodeRefInner::MemNode(node_arc) => {
    //                             match Arc::try_unwrap(node_arc) {
    //                                 Ok(_) => {}
    //                                 Err(_) => {}
    //                             }
    //                             todo!()
    //                         }
    //                         NodeRefInner::DiskNode { .. } => {
    //                             todo!()
    //                         }
    //                     }
    //                 }
    //                 Err(inner_arc) => {
    //                     if let Some(node_arc) = NodeRef::read_inner(&inner_arc, node_store)? {
    //                         Ok(Some((*node_arc).clone()))
    //                     } else {
    //                         Ok(None)
    //                     }
    //                 }
    //             }
    //         }
    //         NodeRef::Empty => Ok(None)
    //     }
    // }

    pub fn save(&self, node_store: &mut dyn NodeStore<N>) -> Result<Option<N::Ptr>> {
        match self {
            NodeRef::Inner(inner) => {
                match inner.write() {
                    Ok(mut node_ref) => {
                        match node_ref.deref() {
                            NodeRefInner::MemNode(node) => {
                                let ptr = node_store.create(node)?;
                                *node_ref = NodeRefInner::DiskNode {
                                    disk_pointer: ptr.clone(),
                                    cached: Arc::downgrade(node),
                                };
                                Ok(Some(ptr))
                            }
                            NodeRefInner::DiskNode { disk_pointer, .. } => {
                                let _ = node_store.inc_ref_count(disk_pointer)?;
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

    pub fn try_update(&self, _node_store: &mut dyn NodeStore<N>) -> Result<Option<N::Ptr>> {
        Err(anyhow!("not implemented"))
    }
}

#[derive(Clone)]
pub struct NullNodeStore {}

impl<N: Node> NodeStore<N> for NullNodeStore {
    fn create(&mut self, _node: &N) -> Result<N::Ptr> {
        Err(anyhow!("not implemented"))
    }

    fn read(&self, _ptr: &N::Ptr) -> Result<Arc<N>> {
        Err(anyhow!("not implemented"))
    }

    fn try_update(&mut self, _ptr: &N::Ptr, _node: &N) -> Result<Option<N::Ptr>> {
        Err(anyhow!("not implemented"))
    }

    fn delete(&mut self, _ptr: &N::Ptr) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn inc_ref_count(&mut self, _ptr: &N::Ptr) -> Result<u32> {
        Ok(1)
    }

    fn dec_ref_count(&mut self, _ptr: &N::Ptr) -> Result<u32> {
        Ok(1)
    }
}
