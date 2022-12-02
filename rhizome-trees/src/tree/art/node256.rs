use crate::tree::art::header::Header;
use crate::tree::art::leaf::Leaf;
use crate::tree::art::node::Node;
use crate::tree::node_manager::{NodeRef, NodeStore};
use crate::tree::traits::{SimpleType, Hashable};

#[derive(Debug, Clone)]
pub(crate) struct Node256<Ptr: SimpleType, V: Hashable> {
    header: Header<Ptr, V>,
    pointers: [NodeRef<Node<Ptr, V>>; 256],
}

impl<Ptr: SimpleType, V: Hashable> Node256<Ptr, V> {
    pub(crate) fn get(&self, key: &[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Option<V>> {
        match self.header.get(key, node_store)? {
            Ok(res) => Ok(res),
            Err(not_found) => {
                match not_found {
                    None => Ok(None),
                    Some(key_rest) => {
                        match self.pointers[key_rest[0] as usize].read(node_store)? {
                            None => Ok(None),
                            Some(node) => {
                                node.get(&key_rest[1..], node_store)
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn insert(&self, key: &[u8], value: V, node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Node<Ptr, V>> {
        let mut new_node = self.clone();
        if key.len() == 0 {
            new_node.header.leaf = NodeRef::new(Node::Leaf(Leaf { prefix: Vec::new(), value }));
        } else {
            let i = key[0] as usize;
            new_node.pointers[i] = NodeRef::new(match new_node.pointers[i].read(node_store)? {
                None => {
                    new_node.header.num_keys += 1;
                    Node::Leaf(Leaf { prefix: Vec::from(&key[1..]), value })
                }
                Some(child) => {
                    child.insert(&key[1..], value, node_store)?
                }
            });
        }

        Ok(Node::Node256(new_node))
    }

    pub(crate) fn delete(&self, key: &[u8], node_store: &dyn NodeStore<Node<Ptr, V>>) -> anyhow::Result<Option<Node<Ptr, V>>> {
        todo!()
        // if key.len() == 0 {
        //     if self.header.leaf.is_empty() {
        //         todo!("doesn't exist do nothing")
        //     }
        //     let mut new_node = self.clone();
        //     new_node.header.leaf = NodeRef::Empty;
        //     Ok(Some(Node::Node256(new_node)))
        // } else {
        //     let i = key[0] as usize;
        //     match self.pointers[i].read(node_store)? {
        //         None => {
        //             todo!("doesn't exist do nothing")
        //         }
        //         Some(child) => {
        //             let new_num_keys = self.header.num_keys - 1;
        //             if new_num_keys == 48 {
        //                 todo!()
        //             } else {
        //                 todo!()
        //             }
        //         }
        //     }
        // }
    }
}