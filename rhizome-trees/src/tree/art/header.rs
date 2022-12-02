use std::borrow::Borrow;
use anyhow::anyhow;
use crate::tree::art::node::Node;
use crate::tree::node_manager::{NodeRef, NodeStore};
use crate::tree::traits::{SimpleType, Hashable};

#[derive(Debug, Clone)]
pub(crate) struct Header<Ptr: SimpleType, V: Hashable> {
    pub num_keys: u8,
    pub prefix: Vec<u8>,
    pub leaf: NodeRef<Node<Ptr, V>>,
}

impl<Ptr: SimpleType, V: Hashable> Header<Ptr, V> {
    pub fn get<'a>(&self, key: &'a[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Result<Option<V>, Option<&'a[u8]>>> {
        let prefix: &[u8] = self.prefix.borrow();
        let prefix_len = prefix.len();
        let key_len = key.len();
        if key_len < prefix_len || key[..prefix_len] != *prefix {
            Ok(Err(None))
        } else if key_len == prefix_len {
            match self.leaf.read(node_store)? {
                None => { Ok(Ok(None)) }
                Some(leaf) => {
                    match leaf.borrow() {
                        Node::Leaf(leaf) => {
                            Ok(Ok(Some(leaf.value.clone())))
                        }
                        node => Err(anyhow!("expected leaf node, got: {:?}", node))
                    }
                }
            }
        } else {
            Ok(Err(Some(&key[prefix_len..])))
        }
    }
}
