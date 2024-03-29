use crate::reach_data::ReachData;
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::hash::Hash;

pub struct BestTimes<T: Hash + Copy + PartialEq + Eq + Serialize> {
    inner: FxHashMap<T, ReachData>,
}

impl<T: Hash + Copy + PartialEq + Eq + Serialize> BestTimes<T> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn new() -> Self {
        Self {
            inner: FxHashMap::default(),
        }
    }
    fn add(&mut self, key: T, data: ReachData) {
        self.inner.insert(key, data);
    }

    pub fn get_mut(&mut self, key: &T) -> Option<&mut ReachData> {
        self.inner.get_mut(key)
    }

    pub fn get(&self, key: &T) -> Option<&ReachData> {
        self.inner.get(key)
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }
    pub fn set_best_time(&mut self, node: T, reach_data: ReachData) -> bool {
        match self.get_mut(&node) {
            Some(x) if x.timestamp > reach_data.timestamp => {
                *x = reach_data;
                true
            }
            None => {
                self.add(node, reach_data);
                true
            }
            _ => false,
        }
    }
}
