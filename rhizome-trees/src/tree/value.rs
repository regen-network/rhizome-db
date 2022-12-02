use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use crate::tree::traits::{Hasher, Hashable};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytesValue(Vec<u8>);

impl Display for BytesValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}

impl Hashable for BytesValue {
    fn hash(&self, digest: &mut dyn Hasher) -> anyhow::Result<()> {
        digest.update(&self.0);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Int32BigEndian(pub i32);

impl Display for Int32BigEndian {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Hashable for Int32BigEndian {
    fn hash(&self, digest: &mut dyn Hasher) -> anyhow::Result<()> {
        digest.update(&self.0.to_be_bytes());
        Ok(())
    }
}

impl From<i32> for Int32BigEndian {
    fn from(x: i32) -> Self {
        Int32BigEndian(x)
    }
}
