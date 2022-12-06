use std::borrow::{Borrow, BorrowMut};
use std::cmp::{max, Ordering};
use std::fmt::Debug;
use std::mem::take;
use std::ops::{Deref};
use std::sync::{Arc, RwLock};
use anyhow::anyhow;

use graphviz_rust::dot_structures::{NodeId};
use crate::hash::HashRoot;
use crate::tree::node_manager::node_ref::{NodeRef};
use crate::tree::node_manager::NodeManager;

use crate::tree::traits::{Hasher, Hashable, MerkleTree, SimpleType, Map, PersistentMap, MutableMap};
use crate::visualization::TreeGraph;

/// A persistent AVL tree.
#[derive(Default, Clone)]
pub struct Tree<K: Ord + SimpleType, V: SimpleType, Ptr: SimpleType + Eq + std::hash::Hash>
{
    root: NodeRef<Node<K, V, Ptr>>,
    node_mgr: NodeManager<Node<K, V, Ptr>>,
}


impl<K: Ord + Hashable, V: Hashable, Ptr: SimpleType + Eq + std::hash::Hash> Map<K, V> for Tree<K, V, Ptr> {
}

impl<K: Ord + Hashable, V: Hashable, Ptr: SimpleType + Eq + std::hash::Hash> PersistentMap<K,V> for Tree<K, V, Ptr> {
    fn insert(&self, key: K, value: V) -> anyhow::Result<Self> {
        let mut res = (*self).clone();
        res.root = Node::insert(res.root, key, value, &self.node_mgr, false)?;
        Ok(res)
    }

    fn delete(&self, key: &K) -> anyhow::Result<Self> {
        // TODO cover case when value doesn't exist and don't copy
        let mut res = (*self).clone();
        res.root = Node::delete(res.root, key, &self.node_mgr, false)?;
        Ok(res)
    }
}

impl<K: Ord + Hashable, V: Hashable, Ptr: SimpleType + Eq + std::hash::Hash> MutableMap<K,V> for Tree<K, V, Ptr> {
    fn insert_mut(&mut self, key: K, value: V) -> anyhow::Result<()> {
        let root = take(&mut self.root);
        self.root = Node::insert(root, key, value, &self.node_mgr, true)?;
        Ok(())
    }


    fn delete_mut(&mut self, key: &K) -> anyhow::Result<()> {
        let root = take(&mut self.root);
        self.root = Node::delete(root, key, &self.node_mgr, true)?;
        Ok(())
    }
}

impl<K: Ord + Hashable, V: Hashable, Ptr: SimpleType + Eq + std::hash::Hash> Tree<K, V, Ptr> {
    fn get<Q: ?Sized>(&self, key: &Q) -> anyhow::Result<Option<V>>
        where
            K: Borrow<Q>,
            Q: Ord {
        match self.node_mgr.read(&self.root)? {
            None => Ok(None),
            Some(r) => r.get(key, &self.node_mgr),
        }
    }

    fn balanced(&self) -> anyhow::Result<bool> {
        let bf = self.balance_factor()?;
        Ok((-1..=1).contains(&bf))
    }

    fn balance_factor(&self) -> anyhow::Result<i32> {
        match &self.node_mgr.read(&self.root)? {
            None => Ok(0),
            Some(root) => root.balance_factor(&self.node_mgr)
        }
    }

    fn to_graphviz(&self, new_hash: fn() -> Box<dyn Hasher>) -> anyhow::Result<TreeGraph> {
        let mut root_hash = HashRoot::new(new_hash);
        self.merkle_hash(&mut root_hash)?;
        let mut graph = TreeGraph::new(hex::encode(root_hash.result));
        if let Some(root) = self.node_mgr.read(&self.root)? {
            root.to_graphviz(&mut graph, &self.node_mgr, new_hash)?;
        }
        Ok(graph)
    }

    fn save(&mut self) -> anyhow::Result<()> {
        if let Some(root) = self.node_mgr.read(&self.root)? {
            root.save_children(self.node_mgr.borrow_mut())?;
            self.node_mgr.save(&self.root)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Node<K: Ord + SimpleType, V: SimpleType, Ptr: SimpleType + Eq + std::hash::Hash> {
    key: K,
    value: V,
    left: NodeRef<Self>,
    right: NodeRef<Self>,
    height: i32,
    hash: RwLock<Option<Vec<u8>>>,
}

impl<K: Ord + Hashable, V: Hashable, Ptr: SimpleType + Eq + std::hash::Hash> Node<K, V, Ptr> {
    fn get<Q: ?Sized>(&self, key: &Q, node_mgr: &NodeManager<Self>) -> anyhow::Result<Option<V>>
        where K: Borrow<Q>, Q: Ord {
        match key.cmp(self.key.borrow()) {
            Ordering::Less => match node_mgr.read(&self.left)? {
                None => Ok(None),
                Some(left) => left.get(key, node_mgr),
            }
            Ordering::Equal => Ok(Some(self.value.clone())),
            Ordering::Greater => match &node_mgr.read(&self.right)? {
                None => Ok(None),
                Some(right) => right.get(key, node_mgr),
            }
        }
    }

    fn insert(node_ref: NodeRef<Self>, key: K, value: V, node_mgr: &NodeManager<Self>, editable: bool) -> anyhow::Result<NodeRef<Self>> {
        Ok(NodeRef::new(match node_mgr.take_or_clone(node_ref, editable)? {
            None => Node::new_node(key, value),
            Some((node, editable)) => {
                node.do_insert(key, value, node_mgr, editable)?
            }
        }))
    }

    fn do_insert(mut self, key: K, value: V, node_mgr: &NodeManager<Self>, editable: bool) -> anyhow::Result<Self> {
        match key.cmp(&self.key) {
            Ordering::Less => {
                self.left = Node::insert(self.left, key, value, node_mgr, editable)?;
            }
            Ordering::Equal => { self.value = value }
            Ordering::Greater => {
                self.right = Node::insert(self.right, key, value, node_mgr, editable)?;
            }
        }

        self.balance(node_mgr)
    }

    fn delete(node_ref: NodeRef<Self>, key: &K, node_mgr: &NodeManager<Self>, editable: bool) -> anyhow::Result<NodeRef<Self>> {
        Ok(match node_mgr.take_or_clone(node_ref, editable)? {
            None => NodeRef::Empty,
            Some((node, editable)) => {
                match node.do_delete(key, node_mgr, editable)? {
                    None => NodeRef::Empty,
                    Some(node) => NodeRef::new(node)
                }
            }
        })
    }

    fn do_delete(mut self, key: &K, node_mgr: &NodeManager<Self>, editable: bool) -> anyhow::Result<Option<Self>> {
        match key.cmp(&self.key) {
            Ordering::Less => {
                self.left = Node::delete(self.left, key, node_mgr, editable)?;
            },
            Ordering::Equal => return Ok(None),
            Ordering::Greater => {
                self.right = Node::delete(self.right, key, node_mgr, editable)?;
            },
        };

        let node = self.balance(node_mgr)?;
        Ok(Some(node))
    }

    fn balance(mut self, node_mgr: &NodeManager<Self>) -> anyhow::Result<Self> {
        let bf = self.update_height(node_mgr)?;
        if bf < -1 { // right big
            if node_mgr.read(&self.right)?.unwrap().balance_factor(node_mgr)? > 0 { // left heavy
                self.rotate_right_left(node_mgr)
            } else {
                self.rotate_left(node_mgr)
            }
        } else if bf > 1 { // left big
            if node_mgr.read(&self.left)?.unwrap().balance_factor(node_mgr)? < 0 { // right heavy
                self.rotate_left_right(node_mgr)
            } else {
                self.rotate_right(node_mgr)
            }
        } else {
            Ok(self)
        }
    }

    // returns the balance factor
    fn update_height(&mut self, node_mgr: &NodeManager<Self>) -> anyhow::Result<i32> {
        let lh = Node::get_height(&node_mgr.read(&self.left)?);
        let rh = Node::get_height(&node_mgr.read(&self.right)?);
        self.height = max(&lh, &rh) + 1;
        Ok(lh - rh)
    }

    fn balance_factor(&self, node_mgr: &NodeManager<Self>) -> anyhow::Result<i32> {
        let lh = Node::get_height(&node_mgr.read(&self.left)?);
        let rh = Node::get_height(&node_mgr.read(&self.right)?);
        Ok(lh - rh)
    }

    fn rotate_right(&self, node_mgr: &NodeManager<Self>) -> anyhow::Result<Self> {
        let left = node_mgr.read(&self.left)?.unwrap(); // safe to unwrap because we know left is Some
        let left_right = left.right.clone();
        let mut new_top = (*left).clone();
        let mut new_right = self.clone();
        new_right.left = left_right;
        new_right.update_height(node_mgr)?;
        new_top.right = NodeRef::new(new_right);
        new_top.update_height(node_mgr)?;
        Ok(new_top)
    }

    fn rotate_left(&self, node_mgr: &NodeManager<Self>) -> anyhow::Result<Self> {
        let right = node_mgr.read(&self.right)?.unwrap(); // safe to unwrap because we know right is Some
        let right_left = right.left.clone();
        let mut new_top = (*right).clone();
        let mut new_left = self.clone();
        new_left.right = right_left;
        new_left.update_height(node_mgr)?;
        new_top.left = NodeRef::new(new_left);
        new_top.update_height(node_mgr)?;
        Ok(new_top)
    }

    fn rotate_right_left(&self, node_mgr: &NodeManager<Self>) -> anyhow::Result<Self> {
        let right = node_mgr.read(&self.right)?.unwrap(); // safe to unwrap because we know right is Some
        let mut new_top = right.rotate_right(node_mgr)?;
        let mut new_left = self.clone();
        new_left.right = new_top.left;
        new_left.update_height(node_mgr)?;
        new_top.left = NodeRef::new(new_left);
        new_top.update_height(node_mgr)?;
        Ok(new_top)
    }

    fn rotate_left_right(&self, node_mgr: &NodeManager<Self>) -> anyhow::Result<Self> {
        let left = node_mgr.read(&self.left)?.unwrap(); // safe to unwrap because we know left is Some
        let mut new_top = left.rotate_left(node_mgr)?;
        let mut new_right = self.clone();
        new_right.left = new_top.right;
        new_right.update_height(node_mgr)?;
        new_top.right = NodeRef::new(new_right);
        new_top.update_height(node_mgr)?;
        Ok(new_top)
    }

    fn new_node(key: K, value: V) -> Self {
        Node {
            key,
            left: NodeRef::Empty,
            right: NodeRef::Empty,
            height: 1,
            value,
            hash: Default::default(),
        }
    }

    fn to_graphviz(&self, graph: &mut TreeGraph, node_mgr: &NodeManager<Self>, new_hash: fn() -> Box<dyn Hasher>) -> anyhow::Result<NodeId> {
        let mut root_hash = HashRoot::new(new_hash);
        self.merkle_hash(node_mgr, &mut root_hash)?;
        let hash_str = hex::encode(&root_hash.result[..8]);
        let id = graph.new_node(format!("{} v={} h={} {}", self.key, self.value, self.height, hash_str));

        if let Some(left) = node_mgr.read(&self.left)? {
            let lid = left.to_graphviz(graph, node_mgr, new_hash)?;
            graph.draw_edge(&id, &lid);
        }

        if let Some(right) = node_mgr.read(&self.right)? {
            let rid = right.to_graphviz(graph, node_mgr, new_hash)?;
            graph.draw_edge(&id, &rid);
        }

        Ok(id)
    }

    fn get_height(mnode: &Option<Arc<Self>>) -> i32 {
        if let Some(node) = mnode {
            node.height
        } else {
            0
        }
    }

    fn save_children(&self, node_mgr: &mut NodeManager<Self>) -> anyhow::Result<()> {
        if let Some(left) = node_mgr.read(&self.left)? {
            left.save_children(node_mgr)?;
        }
        if let Some(right) = node_mgr.read(&self.right)? {
            right.save_children(node_mgr)?;
        }
        node_mgr.save(&self.left)?;
        node_mgr.save(&self.right)?;
        Ok(())
    }

    fn merkle_hash(&self, node_mgr: &NodeManager<Self>, digest: &mut dyn Hasher) -> anyhow::Result<()> {
        match self.hash.read() {
            Err(e) => {
                return Err(anyhow!("poison error: {:?}", e));
            }
            Ok(guard) => {
                if let Some(hash) = guard.deref() {
                    digest.update(hash.as_slice());
                    return Ok(());
                }
            }
        }

        match self.hash.write() {
            Err(err) => {
                Err(anyhow!("poison error: {:?}", err))
            }
            Ok(mut guard) => {
                let mut node_digest = digest.new();
                self.key.hash(&mut *node_digest)?;
                self.value.hash(&mut *node_digest)?;
                let mut have_left = false;
                if let Some(left) = node_mgr.read(&self.left)? {
                    left.merkle_hash(node_mgr, &mut *node_digest)?;
                    have_left = true;
                }
                if let Some(right) = node_mgr.read(&self.right)? {
                    if !have_left {
                        // pad with a 0 in case left is empty to distinguish left from right
                        node_digest.update(&[0]);
                    }
                    right.merkle_hash(node_mgr, &mut *node_digest)?;
                }
                let hash = node_digest.finalize();
                digest.update(hash.borrow());
                *guard = Some(hash);
                Ok(())
            }
        }
    }
}

impl<K: Ord + Hashable, V: Hashable, Ptr: SimpleType + Eq + std::hash::Hash> MerkleTree for Tree<K, V, Ptr> {
    fn merkle_hash(&self, digest: &mut dyn Hasher) -> anyhow::Result<()> {
        match self.node_mgr.read(&self.root)? {
            None => {
                // TODO hash of empty tree
                Ok(())
            }
            Some(root) => {
                root.merkle_hash(&self.node_mgr, digest)
            }
        }
    }
}

impl<K: Ord + SimpleType, V: SimpleType, Ptr: SimpleType + Eq + std::hash::Hash> Clone for Node<K, V, Ptr> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            height: self.height,
            hash: Default::default(),
        }
    }
}

impl<K: Ord + SimpleType, V: SimpleType, Ptr: SimpleType + Eq + std::hash::Hash> crate::tree::node_manager::node_ref::Node for Node<K, V, Ptr> {
    type Ptr = Ptr;
}

#[cfg(test)]
mod tests {
    use crate::tree::avl::Tree;
    use crate::hash::{Blake3Hash};
    use crate::tree::traits::{Hasher, MutableMap, PersistentMap};
    use crate::tree::value::Int32BigEndian;

    #[test]
    fn test_tree() -> anyhow::Result<()> {
        let mut tree: Tree<Int32BigEndian, Int32BigEndian, i32> = Tree::default();
        let mut i = 0;
        let new_hash: fn() -> Box<dyn Hasher> = || Box::new(Blake3Hash::new());
        while i < 10 {
            tree = tree.insert(Int32BigEndian(i), Int32BigEndian(i))?;
            assert!(tree.balanced()?);
            assert_eq!(Some(Int32BigEndian(i)), tree.get(&Int32BigEndian(i))?);
            let graph = tree.to_graphviz(new_hash)?;
            graph.save_file(format!("avl-insert-{:?}.dot", i));
            i += 1
        }

        while i >= 0 {
            tree = tree.delete(&Int32BigEndian(i))?;
            assert_eq!(None, tree.get(&Int32BigEndian(i))?);
            assert!(tree.balanced()?);
            let graph = tree.to_graphviz(new_hash)?;
            graph.save_file(format!("avl-delete-{:?}.dot", i));
            i -= 1
        }

        Ok(())
    }

    #[test]
    fn test_tree_mut() -> anyhow::Result<()> {
        let mut tree: Tree<Int32BigEndian, Int32BigEndian, i32> = Tree::default();
        let mut i = 0;
        while i < 10 {
            tree.insert_mut(Int32BigEndian(i), Int32BigEndian(i))?;
            assert!(tree.balanced()?);
            assert_eq!(Some(Int32BigEndian(i)), tree.get(&Int32BigEndian(i))?);
            i += 1
        }

        Ok(())
    }
}
