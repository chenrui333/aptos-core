// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hash;

pub mod caches;

/// A trait for a cache that can be used to store key-value pairs.
pub trait Cache<K, V>: Send + Sync
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Get the value for a given key. Return [`None`] if the key is not in the cache.
    fn get(&self, key: &K) -> Option<V>;

    /// Inserts a given key-value pair in cache. Panics if the insert fails.
    fn insert(&mut self, key: K, value: V);

    /// Returns the first key in the cache. Returns [`None`] if the cache is empty.
    fn first_key(&self) -> Option<K>;

    /// Returns the last key in the cache. Returns [`None`] if the cache is empty.
    fn last_key(&self) -> Option<K>;
}
