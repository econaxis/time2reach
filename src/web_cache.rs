use crate::web::RequestId;
use crate::web_app_data::CityAppData;
use lazy_static::lazy_static;
use lru::LruCache;
use serde_json::value::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Mutex;
use warp::reply::Json;

struct CacheValue {
    value: Value,
    request: RequestId,
}

impl CacheValue {
    fn new(value: Value, request: RequestId) -> Self {
        CacheValue { value, request }
    }
}
lazy_static! {
    static ref CACHE: Mutex<LruCache<u64, CacheValue>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(250).unwrap()));
}
fn round_f64_for_hash(x: f64) -> u64 {
    (x * 10000.0).round() as u64
}

fn cache_key(
    lat: f64,
    lng: f64,
    include_agencies: &[String],
    include_modes: &[String],
    start_time: u64,
    max_duration_secs: u64,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write_u64(round_f64_for_hash(lat));
    hasher.write_u64(round_f64_for_hash(lng));

    "AGENCY".hash(&mut hasher);
    for agency in include_agencies {
        agency.hash(&mut hasher);
    }

    "MODE".hash(&mut hasher);
    for mode in include_modes {
        mode.hash(&mut hasher);
    }

    start_time.hash(&mut hasher);
    max_duration_secs.hash(&mut hasher);
    hasher.finish()
}

pub fn check_cache<'a>(
    ad: &CityAppData,
    lat: f64,
    lng: f64,
    include_agencies: &[String],
    include_modes: &[String],
    start_time: u64,
    max_duration_secs: u64,
) -> Result<Json, u64> {
    let mut cache = CACHE.lock().unwrap();
    let hash = cache_key(
        lat,
        lng,
        include_agencies,
        include_modes,
        start_time,
        max_duration_secs,
    );
    cache
        .get(&hash)
        .map(|x| {
            ad.rs_list.write().unwrap().promote(x.request.rs_list_index);
            warp::reply::json(&x.value)
        })
        .ok_or(hash)
}

pub fn insert_cache(cache_key: u64, response: Value, req_id: RequestId) -> Json {
    let mut cache = CACHE.lock().unwrap();
    let cached_response = cache.get_or_insert_mut(cache_key, || CacheValue::new(response, req_id));
    warp::reply::json(&cached_response.value)
}
