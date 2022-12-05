use anyhow::{anyhow, Result};
use std::sync::{Arc};
use crate::tree::node_manager::node_ref::{Node};

pub trait NodeStore<N : Node> {
    fn create(&mut self, node: &N) -> Result<N::Ptr>;
    fn read(&self, ptr: &N::Ptr) -> Result<Arc<N>>;
    fn try_update(&mut self, ptr: &N::Ptr, node: &N) -> Result<Option<N::Ptr>>;
    fn delete(&mut self, ptr: &N::Ptr) -> Result<()>;
    fn inc_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;
    fn dec_ref_count(&mut self, ptr: &N::Ptr) -> Result<u32>;
}

#[derive(Clone)]
pub struct NullNodeStore {}

impl<N: Node> NodeStore<N> for NullNodeStore {
    fn create(&mut self, _node: &N) -> Result<N::Ptr> {
        Err(anyhow!("not implemented"))
    }

    fn read(&self, _ptr: &N::Ptr) -> Result<Arc<N>> {
        Err(anyhow!("not implemented"))
    }

    fn try_update(&mut self, _ptr: &N::Ptr, _node: &N) -> Result<Option<N::Ptr>> {
        Err(anyhow!("not implemented"))
    }

    fn delete(&mut self, _ptr: &N::Ptr) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    fn inc_ref_count(&mut self, _ptr: &N::Ptr) -> Result<u32> {
        Ok(1)
    }

    fn dec_ref_count(&mut self, _ptr: &N::Ptr) -> Result<u32> {
        Ok(1)
    }
}
