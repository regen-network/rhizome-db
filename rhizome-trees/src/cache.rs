pub trait Cache<K, V> {
    fn cache_get(&self, k: &K) -> Option<V>;
}