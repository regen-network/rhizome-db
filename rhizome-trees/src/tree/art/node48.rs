use anyhow::anyhow;
use crate::tree::art::header::Header;
use crate::tree::art::node::Node;
use crate::tree::node_manager::{NodeRef, NodeStore};
use crate::tree::traits::{SimpleType, Hashable};

#[derive(Debug, Clone)]
pub(crate) struct Node48<Ptr: SimpleType, V: Hashable> {
    header: Header<Ptr, V>,
    key_pointer_indices: [u8; 256],
    pointers: [NodeRef<Node<Ptr, V>>; 48],
}

impl<Ptr: SimpleType, V: Hashable> Node48<Ptr, V> {
    pub fn get(&self, key: &[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Option<V>> {
        match self.header.get(key, node_store)? {
            Ok(res) => Ok(res),
            Err(not_found) => {
                match not_found {
                    None => Ok(None),
                    Some(key_rest) => {
                        let key_pointer_idx = self.key_pointer_indices[key_rest[0] as usize];
                        if key_pointer_idx == 0 {
                            Ok(None)
                        } else {
                            match self.pointers[(key_pointer_idx - 1) as usize].read(node_store)? {
                                None => Err(anyhow!("unexpected missing node")),
                                Some(node) => {
                                    node.get(&key_rest[1..], node_store)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
