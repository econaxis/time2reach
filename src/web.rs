use crate::agencies::{agencies, City, load_all_gtfs};
use crate::configuration::Configuration;
use crate::gtfs_setup::get_agency_id_from_short_name;
use crate::gtfs_wrapper::RouteType;
use crate::road_structure::{EdgeId};
use crate::{Gtfs1, gtfs_setup, RoadStructure, Time, time_to_reach, trip_details};
use lazy_static::lazy_static;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;

use lru::LruCache;
use std::sync::{Arc, Mutex};

use warp::{Filter, Reply};
use std::num::NonZeroUsize;
use crate::trip_details::CalculateRequest;
use crate::web_app_data::{AllAppData, CityAppData};

lazy_static! {
    pub static ref CACHE: Mutex<LruCache<u64, Value>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(15).unwrap()));
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
    hasher.finish()
}

fn gtfs_to_city_appdata(city: City, gtfs: Gtfs1) -> Arc<Mutex<CityAppData>> {
    let data = gtfs_setup::generate_stops_trips(&gtfs).into_spatial(&gtfs);

    CityAppData::new(gtfs, data, city)
}
fn check_cache<'a>(
    cache: &'a mut LruCache<u64, Value>,
    lat: f64,
    lng: f64,
    include_agencies: &[String],
    include_modes: &[String],
    start_time: u64,
) -> Result<&'a Value, u64> {
    let hash = cache_key(lat, lng, include_agencies, include_modes, start_time);
    cache.get(&hash).ok_or(hash)
}

fn check_city(ad: &Arc<AllAppData>, lat: f64, lng: f64) -> Option<City> {
    for (city, data) in &ad.ads {
        let data = data.lock().unwrap();
        let is_near_point = data.spatial.is_near_point(LatLng {
            latitude: lat,
            longitude: lng,
        });

        if is_near_point {
            return Some(*city);
        }
    }
    None
}

fn process_coordinates(
    ad: Arc<AllAppData>,
    lat: f64,
    lng: f64,
    include_agencies: Vec<String>,
    include_modes: Vec<String>,
    start_time: u64,
    prev_request_id: Option<RequestId>,
) -> impl Reply {
    let city = check_city(&ad, lat, lng);

    if city.is_none() {
        return warp::reply::json(&"No city found nearby");
    }
    let city = city.unwrap();
    let ad = &ad.ads.get(&city).unwrap();
    let mut ad = ad.lock().unwrap();
    let ad = ad.deref_mut();

    if let Some(req) = prev_request_id {
        ad.rs_list.remove(req.rs_list_index);
    }

    let mut cache = CACHE.lock().unwrap();

    let cache_key = match check_cache(
        &mut cache,
        lat,
        lng,
        &include_agencies,
        &include_modes,
        start_time,
    ) {
        Ok(reply) => return warp::reply::json(reply),
        Err(key) => key,
    };

    let gtfs = &ad.gtfs;
    let spatial_stops = &ad.spatial;
    let rs_template = ad.rs_template.clone();
    let mut rs = RoadStructure::new_from_road_structure(rs_template);

    let agency_ids: HashSet<u8> = include_agencies
        .iter()
        .map(|ag| get_agency_id_from_short_name(ag))
        .collect();

    let modes = include_modes
        .iter()
        .map(|x| RouteType::from(x.as_ref()))
        .collect();

    time_to_reach::generate_reach_times(
        gtfs,
        spatial_stops,
        &mut rs,
        Configuration {
            start_time: Time(start_time as f64),
            duration_secs: 3600.0 * 1.5,
            location: LatLng {
                latitude: lat,
                longitude: lng,
            },
            agency_ids,
            modes,
        },
    );

    let edge_times = rs.save();
    let edge_times_object: HashMap<EdgeId, u32> = edge_times
        .into_iter()
        .map(|edge_time| (edge_time.edge_id, edge_time.time as u32))
        .collect();

    let rs_list_index = ad.rs_list.push(rs);
    let request_id = RequestId {
        rs_list_index,
        city,
    };
    let response = json!({
        "request_id": request_id,
        "edge_times": edge_times_object
    });

    let cached_response = cache.get_or_insert_mut(cache_key, || response);
    warp::reply::json(cached_response)
}

#[derive(Deserialize, Clone, Copy)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

impl LatLng {
    pub fn from_lat_lng(lat: f64, lng: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lng,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RequestId {
    pub rs_list_index: usize,
    pub city: City,
}

fn with_appdata(
    ad: Arc<AllAppData>,
) -> impl Filter<Extract = (Arc<AllAppData>,), Error = Infallible> + Clone {
    warp::any().map(move || ad.clone())
}

pub async fn main() {
    let all_gtfs = load_all_gtfs();
    let all_gtfs: HashMap<City, Arc<Mutex<CityAppData>>> = all_gtfs
        .into_iter()
        .map(|(city, gtfs)| (city, gtfs_to_city_appdata(city, gtfs)))
        .collect();

    let appdata = Arc::new(AllAppData { ads: all_gtfs });

    let cors_policy = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Origin",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(["POST", "GET"]);

    info!("Setup done");

    let log = warp::log("warp");
    let hello = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path!("hello"))
        .and(warp::body::json())
        .map(|ad: Arc<AllAppData>, req: CalculateRequest| {
            process_coordinates(
                ad,
                req.latitude,
                req.longitude,
                req.agencies,
                req.modes,
                req.start_time,
                req.previous_request_id,
            )
        });

    let details = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path!("details"))
        .and(warp::body::json())
        .map(trip_details::get_trip_details);

    let agencies_endpoint = warp::get()
        .and(warp::path!("agencies"))
        .map(|| warp::reply::json(&agencies()));

    let routes = hello
        .or(details)
        .or(agencies_endpoint)
        .with(cors_policy)
        .with(log);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
