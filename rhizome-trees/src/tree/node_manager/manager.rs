use std::ops::Deref;
use std::sync::{Arc, LockResult, RwLock, Weak};
use anyhow::anyhow;
use crate::tree::node_manager::Node;

#[derive(Debug, Clone)]
pub enum NodeRef<N: Node>
{
    Inner(Arc<RwLock<NodeRefInner<N>>>),
    Empty,
}

#[derive(Debug, Clone)]
pub enum NodeRefInner<N: Node> {
    MemNode(N),
    DiskNode { disk_pointer: N::Ptr, cached: Weak<N> },
}

impl <N:Node> NodeRef<N> {
    // fn get(&self) -> anyhow::Result<Option<&dyn Deref<Target = N>>>
    // {
    //     match self {
    //         NodeRef::Inner(inner) => {
    //             match inner.read() {
    //                 Ok(inner) => {
    //                     match *inner {
    //                         NodeRefInner::MemNode(_) => {
    //                             todo!()
    //                         }
    //                         NodeRefInner::DiskNode { .. } => {
    //                             todo!()
    //                         }
    //                     }
    //                 }
    //                 Err(_) => Err(anyhow!("poison error"))
    //             }
    //         }
    //         NodeRef::Empty => Ok(None),
    //     }
    // }

    fn take_or_clone(self) -> anyhow::Result<Option<N>> {
            match self {
                NodeRef::Inner(inner) => {
                    match Arc::try_unwrap(inner) {
                        Ok(inner) => {
                            match RwLock::into_inner(inner) {
                                Ok(inner) => {
                                    match inner {
                                        NodeRefInner::MemNode(n) => Ok(Some(n)),
                                        NodeRefInner::DiskNode { .. } => {
                                            todo!()
                                        }
                                    }
                                }
                                Err(_) => {
                                    todo!()
                                }
                            }
                        }
                        Err(_) => {
                            todo!()
                        }
                    }
                }
                NodeRef::Empty => Ok(None),
            }
    }
}
