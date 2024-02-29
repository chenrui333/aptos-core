// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::Cache;
use quick_cache::{sync::Cache as S3FIFOCache, Lifecycle, Weighter};
use std::hash::{BuildHasher, Hash};

impl<K, V, We, B, L> Cache<K, V> for S3FIFOCache<K, V, We, B, L>
where
    K: Eq + Hash + Clone,
    V: Clone,
    We: Weighter<K, V> + Clone,
    B: BuildHasher + Clone,
    L: Lifecycle<K, V> + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        S3FIFOCache::get(&self, key)
    }

    fn insert(&mut self, key: K, value: V) -> anyhow::Result<()> {
        Ok(S3FIFOCache::insert(&self, key, value))
    }
}
