use crate::gtfs_processing::SpatialStopsWithTrips;
use crate::road_structure::RoadStructureInner;
use crate::{City, Gtfs1, RoadStructure};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, RwLock};

pub struct CityAppData {
    pub gtfs: Gtfs1,
    pub spatial: SpatialStopsWithTrips,
    pub rs_template: Arc<RoadStructureInner>,
    pub rs_list: RwLock<RoadStructureList>,
}

pub struct AllAppData {
    pub ads: HashMap<City, CityAppData>,
}

impl CityAppData {
    pub(crate) fn new1(gtfs: Gtfs1, spatial: SpatialStopsWithTrips, city: City) -> CityAppData {
        let rs = RoadStructureInner::new(city);
        CityAppData {
            gtfs,
            spatial,
            rs_template: Arc::new(rs),
            rs_list: RwLock::new(RoadStructureList::new(20)),
        }
    }
}

pub struct RoadStructureList {
    pub inner: LruCache<usize, RoadStructure>,
    pub counter: usize,
}

impl RoadStructureList {
    pub fn remove(&mut self, key: usize) {
        self.inner.pop(&key);
    }
    pub fn push(&mut self, rs: RoadStructure) -> usize {
        self.counter += 1;

        self.inner.put(self.counter, rs);

        self.counter
    }

    pub fn pre_get(&mut self, key: usize) {
        self.inner.get(&key);
    }
    pub fn get(&self, key: usize) -> Option<&RoadStructure> {
        self.inner.peek(&key)
    }

    pub fn new(max: usize) -> Self {
        RoadStructureList {
            inner: LruCache::new(NonZeroUsize::new(max).unwrap()),
            counter: 0,
        }
    }
}
