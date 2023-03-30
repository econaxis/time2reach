use std::hash::Hash;
use std::collections::HashMap;
use serde::{Serialize, Serializer};
use crate::ReachData;

pub struct BestTimes<T: Hash + Copy + PartialEq + Eq + Serialize> {
    inner: HashMap<T, ReachData>
}

impl<T: Hash + Copy + PartialEq + Eq + Serialize> serde::Serialize for BestTimes<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.inner.serialize(serializer)
    }
}

impl<T: Hash + Copy + PartialEq + Eq + Serialize> BestTimes<T> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new()
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
