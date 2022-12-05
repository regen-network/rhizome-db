use std::sync::{Arc, RwLock, Weak};

pub trait Node: Sync + Send + Clone {
    type Ptr: Sync + Send + Clone + std::fmt::Debug + Eq + std::hash::Hash;
}

#[derive(Debug, Clone, Default)]
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

impl<N: Node> NodeRef<N> {
    pub fn new(node: N) -> Self {
        NodeRef::Inner(Arc::new(RwLock::new(NodeRefInner::MemNode(Arc::new(node)))))
    }
}
