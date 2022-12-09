use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

pub trait Node: Sync + Send + Clone {
    type Ptr: Sync + Send + Clone + std::fmt::Debug + Eq + std::hash::Hash;
}

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

impl<N: Node> Default for NodeRef<N> {
    fn default() -> Self {
        NodeRef::Empty
    }
}

impl<N: Node> NodeRef<N> {
    pub fn new(node: N) -> Self {
        NodeRef::Inner(Arc::new(RwLock::new(NodeRefInner::MemNode(node))))
    }
}

pub enum NodeHandle<'a, N: Node> {
    Direct(RwLockReadGuard<'a, NodeRefInner<N>>),
    Arc(Arc<N>),
}

impl<'a, N: Node> Deref for NodeHandle<'a, N> {
    type Target = N;

    fn deref(&self) -> &Self::Target {
        match self {
            NodeHandle::Direct(guard) => match guard.deref() {
                NodeRefInner::MemNode(node) => node,
                _ => panic!("unexpected")
            },
            NodeHandle::Arc(arc) => arc.deref(),
        }
    }
}


