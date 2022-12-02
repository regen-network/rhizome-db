use anyhow::anyhow;
use crate::tree::art::header::Header;
use crate::tree::art::node::Node;
use crate::tree::node_manager::{NodeRef, NodeStore};
use crate::tree::traits::{SimpleType, Hashable};

#[derive(Debug, Clone)]
pub(crate) struct Node16<Ptr: SimpleType, V: Hashable> {
    header: Header<Ptr, V>,
    keys: [u8; 16],
    pointers: [NodeRef<Node<Ptr, V>>; 16],
}

impl<Ptr: SimpleType, V: Hashable> Node16<Ptr, V> {
    pub fn get(&self, key: &[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Option<V>> {
        match self.header.get(key, node_store)? {
            Ok(res) => Ok(res),
            Err(not_found) => {
                match not_found {
                    None => Ok(None),
                    Some(key_rest) => {
                        let key0 = key_rest[0];
                        for i in 0..self.header.num_keys as usize {
                            // TODO simd
                            if self.keys[i] == key0 {
                                return match self.pointers[i].read(node_store)? {
                                    None => Err(anyhow!("unexpected missing node")),
                                    Some(node) => {
                                        node.get(&key_rest[1..], node_store)
                                    }
                                };
                            }
                        }
                        Ok(None)
                    }
                }
            }
        }
    }
}
