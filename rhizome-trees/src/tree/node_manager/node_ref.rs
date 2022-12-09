//! The [NodeRef] and [NodeHandle] primitives

use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

/// The trait that all tree nodes should implement.
pub trait Node: Sync + Send + Clone {
    /// The type used for serializing tree nodes to storage layers.
    /// This should normally parameterized at the node level to allow for
    /// different types of node pointers.
    type Ptr: Sync + Send + Clone + std::fmt::Debug + Eq + std::hash::Hash;
}

/// A reference to a tree node in memory or in storage.
#[derive(Debug, Clone)]
pub enum NodeRef<N: Node>
{
    /// Internal node data.
    Inner(Arc<RwLock<NodeRefInner<N>>>),
    /// An empty node.
    Empty,
}

/// Internal node data. TODO: find a way to make this private.
#[derive(Debug, Clone)]
pub enum NodeRefInner<N: Node> {
    /// A node stored in memory.
    MemNode(N),
    /// A node stored on a storage medium.
    StoredNode {
        /// a pointer to the node in storage.
        pointer: N::Ptr,

        /// a weak reference to the node in the in-memory cache (if it has been loaded from storage).
        cached: Weak<N>
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


