use std::fmt::{Debug, Display};

pub trait SimpleType: Debug + Clone + Sync + Send + Display {}

pub trait Hashable: SimpleType {
    fn hash(&self, digest: &mut dyn Hasher) -> anyhow::Result<()>;
}

impl<T: Debug + Clone + Sync + Send + Display> SimpleType for T {}

pub trait Reader<T> {
    fn read(&self, value: T) -> anyhow::Result<()>;
}

impl <T> Reader<T> for dyn Fn (T) -> anyhow::Result<()> {
    fn read(&self, value: T) -> anyhow::Result<()>{
        self(value)
    }
}

pub trait Hasher {
    fn new(&self) -> Box<dyn Hasher>;
    fn update(&mut self, value: &[u8]);
    fn finalize(&mut self) -> Vec<u8>;
}

pub trait Map<K, V> {
    // fn get<Q: ?Sized>(&self, key: &Q, reader: &dyn Reader<Option<&V>>) -> anyhow::Result<()>
    //     where K: Borrow<Q>, Q: Ord;
}

pub trait PersistentMap<K, V> : Map<K, V> + Sized
    where K: Clone, V: Clone
{
    fn insert(&self, key: K, value: V) -> anyhow::Result<Self>;
    fn delete(&self, key: &K) -> anyhow::Result<Self>;
}

pub trait MutableMap<K, V>: Map<K, V>
    where K: Clone, V: Clone
{
    fn insert_mut(&mut self, key: K, value: V) -> anyhow::Result<()>;
    fn delete_mut(&mut self, key: &K) -> anyhow::Result<()>;
}

pub trait MerkleTree {
    fn merkle_hash(&self, digest: &mut dyn Hasher) -> anyhow::Result<()>;
}
