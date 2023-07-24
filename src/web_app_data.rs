use crate::gtfs_processing::SpatialStopsWithTrips;
use crate::road_structure::RoadStructureInner;
use crate::{City, Gtfs1, RoadStructure};
use lru::LruCache;
use rustc_hash::FxHashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

pub struct CityAppData {
    pub gtfs: Gtfs1,
    pub spatial: SpatialStopsWithTrips,
    pub rs_template: Arc<RoadStructureInner>,
    pub rs_list: RwLock<RoadStructureList>,
}

pub struct AllAppData {
    pub ads: FxHashMap<City, CityAppData>,
}

impl CityAppData {
    pub(crate) fn new1(gtfs: Gtfs1, spatial: SpatialStopsWithTrips, city: City) -> CityAppData {
        let rs = RoadStructureInner::new(city);
        CityAppData {
            gtfs,
            spatial,
            rs_template: Arc::new(rs),
            rs_list: RwLock::new(RoadStructureList::new(250)),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CacheKey(bool, u64);

pub struct RoadStructureList {
    pub inner: LruCache<u64, RoadStructure>,
    pub inner_large: LruCache<u64, RoadStructure>,
    pub counter: u64,
}

impl RoadStructureList {
    fn rs_large(rs: &RoadStructure) -> bool {
        rs.nb.len() > 70000
    }
    fn route_mut(&mut self, key: CacheKey) -> &mut LruCache<u64, RoadStructure> {
        match key.0 {
            false => &mut self.inner,
            true => &mut self.inner_large,
        }
    }
    fn route(&self, key: CacheKey) -> &LruCache<u64, RoadStructure> {
        match key.0 {
            false => &self.inner,
            true => &self.inner_large,
        }
    }
    pub fn push(&mut self, rs: RoadStructure) -> CacheKey {
        self.counter += 1;

        if Self::rs_large(&rs) {
            self.inner_large.put(self.counter, rs);
            CacheKey(true, self.counter)
        } else {
            self.inner.put(self.counter, rs);
            CacheKey(false, self.counter)
        }
    }

    pub fn promote(&mut self, key: CacheKey) {
        self.route_mut(key).promote(&key.1);
    }
    pub fn get(&self, key: CacheKey) -> Option<&RoadStructure> {
        self.route(key).peek(&key.1)
    }

    pub fn new(max: usize) -> Self {
        RoadStructureList {
            inner: LruCache::new(NonZeroUsize::new(max).unwrap()),
            inner_large: LruCache::new(NonZeroUsize::new(20).unwrap()),
            counter: 0,
        }
    }
}
