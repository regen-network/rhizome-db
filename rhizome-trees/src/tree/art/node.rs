use crate::tree::art::leaf::Leaf;
use crate::tree::art::node4::Node4;
use crate::tree::art::node48::Node48;
use crate::tree::art::node16::Node16;
use crate::tree::art::node256::Node256;
use crate::tree::node_manager::NodeStore;
use crate::tree::traits::{SimpleType, Hashable};

#[derive(Debug, Clone)]
pub(crate) enum Node<Ptr: SimpleType, V: Hashable> {
    Leaf(Leaf<V>),
    Node4(Node4<Ptr, V>),
    Node16(Node16<Ptr, V>),
    Node48(Node48<Ptr, V>),
    Node256(Node256<Ptr, V>),
}

impl<Ptr: SimpleType, V: Hashable> crate::tree::node_manager::Node for Node<Ptr, V> {
    type Ptr = Ptr;
}

impl<Ptr: SimpleType, V: Hashable> Node<Ptr, V> {
    pub(crate) fn get(&self, key: &[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Option<V>> {
        match self {
            Node::Leaf(x) => Ok(x.get(key)),
            Node::Node4(x) => x.get(key, node_store),
            Node::Node16(x) => x.get(key, node_store),
            Node::Node48(x) => x.get(key, node_store),
            Node::Node256(x) => x.get(key, node_store),
        }
    }

    pub(crate) fn insert(&self, key: &[u8], value: V, node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Node<Ptr, V>> {
        todo!()
    }

    pub(crate) fn delete(&self, key: &[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Option<Node<Ptr, V>>> {
        // TODO cover case when value doesn't exist and don't copy
        todo!()
    }
}
