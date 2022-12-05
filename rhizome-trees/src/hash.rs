use crate::tree::traits::Hasher;

pub struct Blake3Hash {
    hasher: blake3::Hasher,
}

impl Blake3Hash {
    pub fn new() -> Self {
        Blake3Hash { hasher: blake3::Hasher::new() }
    }
}

impl Hasher for Blake3Hash {
    fn new(&self) -> Box<dyn Hasher> {
        Box::new(Self::new())
    }

    fn update(&mut self, value: &[u8]) {
        self.hasher.update(value);
    }

    fn finalize(&mut self) -> Vec<u8> {
        let hash: [u8; 32] = self.hasher.finalize().into();
        Vec::from(hash)
    }
}

pub struct HashRoot {
    pub new_hash: fn() -> Box<dyn Hasher>,
    pub result: Vec<u8>,
}

impl HashRoot {
    pub fn new(new_hash: fn() -> Box<dyn Hasher>) -> Self {
        HashRoot{
            new_hash,
            result: vec![]
        }
    }
}

impl Hasher for HashRoot {
    fn new(&self) -> Box<dyn Hasher> {
        (self.new_hash)()
    }

    fn update(&mut self, value: &[u8]) {
        self.result = Vec::from(value);
    }

    fn finalize(&mut self) -> Vec<u8> {
        self.result.clone()
    }
}
