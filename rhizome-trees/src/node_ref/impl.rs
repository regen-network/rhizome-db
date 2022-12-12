use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

/// The trait that all tree nodes should implement.
pub trait Node: Sync + Send + Clone {
    /// The type used for serializing tree nodes to storage layers.
    /// This should normally parameterized at the node level to allow for
    /// different types of node pointers.
    type Ptr: Pointer;
}

pub trait Pointer: Sync + Send + Clone + std::fmt::Debug + Eq + std::hash::Hash {}

impl <Ptr> Pointer for Ptr where
    Ptr: Sync + Send + Clone + std::fmt::Debug + Eq + std::hash::Hash {}

/// A reference to a tree node in memory or in storage.
#[derive(Debug, Clone)]
pub enum NodeRef<N: Node>
{
    /// Internal node data.
    Inner(Arc<RwLock<NodeRefInner<N>>>),
    /// An empty node.
    Empty,
}

/// Internal node data.
#[derive(Debug, Clone)]
pub enum NodeRefInner<N: Node> {
    /// A node stored in memory.
    MemNode(N),
    /// A node stored on a storage medium.
    StoredNode {
        /// a pointer to the node in storage.
        pointer: N::Ptr,

        /// a weak reference to the node in the in-memory cache (if it has been loaded from storage).
        cached: Weak<N>,
    },
}

impl<N: Node> Default for NodeRef<N> {
    fn default() -> Self {
        NodeRef::Empty
    }
}

impl<N: Node> NodeRef<N> {
    /// Create a new NodeRef from a node.
    pub fn new(node: N) -> Self {
        NodeRef::Inner(Arc::new(RwLock::new(NodeRefInner::MemNode(node))))
    }

    /// Creates a NodeRef from storage layer pointer.
    pub fn from_ptr(ptr: N::Ptr) -> Self {
        NodeRef::Inner(Arc::new(RwLock::new(NodeRefInner::StoredNode {
            pointer: ptr,
            cached: Weak::default()
        })))
    }
}

/// An in-memory handle to a node that may have been loaded from memory or storage.
pub enum NodeHandle<'a, N: Node> {
    /// A handle to an in-memory node.
    Mem(RwLockReadGuard<'a, NodeRefInner<N>>),

    /// A handle to a node loaded from storage.
    Arc(Arc<N>),
}

impl<'a, N: Node> Deref for NodeHandle<'a, N> {
    type Target = N;

    fn deref(&self) -> &Self::Target {
        match self {
            NodeHandle::Mem(guard) => match guard.deref() {
                NodeRefInner::MemNode(node) => node,
                _ => panic!("unexpected")
            },
            NodeHandle::Arc(arc) => arc.deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use crate::node_ref::{Node, NodeRef};
    use crate::node_ref::NodeRef::Empty;
    use crate::node_ref::r#impl::{NodeRefInner, Pointer};
    use coverage_helper::test;

    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct TestNode<Ptr> {
        pub(crate) ptr: Ptr
    }

    impl <Ptr: Pointer> Node for TestNode<Ptr> {
        type Ptr = Ptr;
    }

    #[test]
    fn test_default() {
        match NodeRef::<TestNode<i64>>::default() {
            NodeRef::Inner(_) => panic!(),
            Empty => {}
        }
    }

    #[test]
    fn test_new() {
        let node = TestNode{
            ptr: 1,
        };
        match NodeRef::new(node.clone()) {
            NodeRef::Inner(inner) => {
                match inner.read() {
                    Ok(guard) => {
                        match guard.deref() {
                            NodeRefInner::MemNode(node2) => {
                                assert_eq!(&node, node2)
                            }
                            NodeRefInner::StoredNode { .. } => {
                                panic!()
                            }
                        }
                    }
                    Err(_) => panic!()
                }
            }
            Empty => panic!()
        }
    }

    #[test]
    fn test_from_pointer() {
        match NodeRef::<TestNode<i64>>::from_ptr(1) {
            NodeRef::Inner(inner) => {
                match inner.read() {
                    Ok(guard) => {
                        match guard.deref() {
                            NodeRefInner::MemNode(_) => panic!(),
                            NodeRefInner::StoredNode { cached, pointer } => {
                                assert_eq!(pointer, &1);
                                assert!(cached.upgrade().is_none());
                            }
                        }
                    }
                    Err(_) => panic!()
                }
            }
            Empty => panic!()
        }
    }
}