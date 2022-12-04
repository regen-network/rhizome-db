pub struct Proof {
    pub hash_alg: HashAlg,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub inner_ops: Vec<InnerOp>,
}

pub struct InnerOp {
    pub prefix: Vec<u8>,
    pub suffix: Vec<u8>,
}

pub enum HashAlg {
    // Sha2_256,
    // Blake2b256,
    Blake3_256,
}

impl Proof {
    fn verify(&self) -> bool {
        todo!()
    }

}

impl HashAlg {
    fn new_hasher(&self) -> Hasher {
        todo!()
    }
}

pub struct Hasher {

}

impl Hasher {
    fn update(&mut self, bytes: &[u8]) {

    }

    fn finalize(&mut self) -> Vec<u8> {
        todo!()
    }
}

struct ProofSpec {
    pub hash_alg: HashAlg,
    pub pre_hash_key: Option<HashAlg>,
    pub pre_hash_val: Option<HashAlg>,
}

impl  ProofSpec {
    fn hash_leaf(&self, prefix: &[u8], key: &[u8], value: &[u8]) -> Vec<u8> {
        let mut hasher = self.hash_alg.new_hasher();
        // hasher.update(varint(key.len()))
        // pre-hash key
        // hasher.update(key)
        // hasher.update(varint(value.len())
        // pre-hash value
        // hasher.update(value)
        hasher.finalize()
    }
}