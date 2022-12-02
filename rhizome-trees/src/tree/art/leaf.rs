use crate::tree::art::node::Node;
use crate::tree::art::util::{cmp_prefix, PrefixMatch};
use crate::tree::traits::{SimpleType, Hashable};

#[derive(Debug, Clone)]
pub(crate) struct Leaf<V> {
    pub(crate) prefix: Vec<u8>,
    pub(crate) value: V,
}

impl <V: Hashable> Leaf<V> {
    pub(crate) fn get(&self, key: &[u8]) -> Option<V> {
        if key == &self.prefix {
            Some(self.value.clone())
        } else {
            None
        }
    }

    pub(crate) fn insert<Ptr: SimpleType>(&self, key: &[u8], value: V) -> anyhow::Result<Node<Ptr, V>> {
        match cmp_prefix(&self.prefix, key) {
            PrefixMatch::Equal => {
                let mut new_node = self.clone();
                new_node.value = value;
                Ok(Node::Leaf(new_node))
            }
            PrefixMatch::PartialMatch(_) => todo!(),
            PrefixMatch::NoMatch => todo!(),
        }
    }
}