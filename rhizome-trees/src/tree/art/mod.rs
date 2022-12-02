use std::fmt::Debug;
use crate::tree::art::node::Node;
use crate::tree::node_manager::{NodeManager, NodeRef};
use crate::tree::traits::{Hashable, SimpleType};

mod node;
mod header;
mod leaf;
mod node4;
mod node16;
mod node48;
mod node256;
mod util;

pub struct Tree<Ptr: SimpleType, V: Hashable> {
    root: NodeRef<Node<Ptr, V>>,
    node_mgr: NodeManager<Node<Ptr, V>>
}