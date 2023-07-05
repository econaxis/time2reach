use crate::agencies::{agencies, load_all_gtfs, City};
use crate::configuration::Configuration;
use crate::gtfs_setup::get_agency_id_from_short_name;
use crate::road_structure::EdgeId;
use crate::{gtfs_setup, time_to_reach, trip_details, Gtfs1, RoadStructure, Time};
use gtfs_structure_2::gtfs_wrapper::RouteType;
use log::info;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

use std::convert::Infallible;

use std::sync::Arc;

use crate::trip_details::CalculateRequest;
use crate::web_app_data::{AllAppData, CityAppData};
use crate::web_cache::{check_cache, insert_cache};
use warp::http::HeaderValue;
use warp::hyper::StatusCode;
use warp::{Filter, Rejection, Reply};

fn gtfs_to_city_appdata(city: City, gtfs: Gtfs1) -> CityAppData {
    let data = gtfs_setup::generate_stops_trips(&gtfs).into_spatial(&gtfs);

    CityAppData::new1(gtfs, data, city)
}

fn check_city(ad: &Arc<AllAppData>, lat: f64, lng: f64) -> Option<City> {
    for (city, data) in &ad.ads {
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

    if let Some(req) = prev_request_id {
        ad.rs_list.write().unwrap().remove(req.rs_list_index);
    }

    let cache_key = match check_cache(lat, lng, &include_agencies, &include_modes, start_time) {
        Ok(reply) => return reply,
        Err(key) => key,
    };

    let gtfs = &ad.gtfs;
    let spatial_stops = &ad.spatial;
    let rs_template = ad.rs_template.clone();
    let mut rs = RoadStructure::new_from_road_structure(rs_template);

    let agency_ids: FxHashSet<u8> = include_agencies
        .iter()
        .filter_map(|ag| get_agency_id_from_short_name(ag))
        .collect();

    let modes = include_modes
        .iter()
        .filter_map(|x| RouteType::try_from(x.as_ref()).ok())
        .collect();

    time_to_reach::generate_reach_times(
        gtfs,
        spatial_stops,
        &mut rs,
        Configuration {
            start_time: Time(start_time as f64),
            duration_secs: 3600.0 * 2.0,
            location: LatLng {
                latitude: lat,
                longitude: lng,
            },
            agency_ids,
            modes,
        },
    );

    let edge_times = rs.save();
    let edge_times_object: FxHashMap<EdgeId, u32> = edge_times
        .into_iter()
        .map(|edge_time| (edge_time.edge_id, edge_time.time as u32))
        .collect();

    let rs_list_index = ad.rs_list.write().unwrap().push(rs);
    let request_id = RequestId {
        rs_list_index,
        city,
    };
    let response = json!({
        "request_id": request_id,
        "edge_times": edge_times_object
    });

    insert_cache(cache_key, response)
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
    let all_gtfs: FxHashMap<City, CityAppData> = all_gtfs
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
        })
        .with(warp::filters::compression::gzip());

    let details = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path!("details"))
        .and(warp::body::json())
        .map(trip_details::get_trip_details)
        .map(|result: Result<warp::reply::Json, &'static str>| {
            result.map(|a| a.into_response()).unwrap_or_else(|err| {
                warp::reply::with_status(err, StatusCode::INTERNAL_SERVER_ERROR).into_response()
            })
        })
        .with(warp::filters::compression::gzip());

    let agencies_endpoint = warp::get()
        .and(warp::path!("agencies"))
        .map(|| warp::reply::json(&agencies()))
        .map(|response: warp::reply::Json| {
            let mut resp = response.into_response();
            let headers = resp.headers_mut();
            headers.append("Cache-Control", HeaderValue::from_static("max-age=18000"));
            resp
        });

    let mvt_endpoint = warp::path("mvt")
        .and(warp::fs::dir("/tmp/vancouver-cache"))
        .map(|response: warp::filters::fs::File| {
            let mut resp = response.into_response();
            let headers = resp.headers_mut();
            headers.append("Content-Encoding", HeaderValue::from_static("gzip"));
            headers.append(
                "Content-Type",
                HeaderValue::from_static("application/x-protobuf"),
            );
            headers.append("Cache-Control", HeaderValue::from_static("max-age=18000"));
            resp
        })
        .recover(|x: Rejection| async {
            if x.is_not_found() {
                Result::<StatusCode, Rejection>::Ok(StatusCode::NO_CONTENT)
            } else {
                Result::<StatusCode, Rejection>::Err(x)
            }
        });

    let routes = hello
        .or(details)
        .or(agencies_endpoint)
        .or(mvt_endpoint)
        .with(cors_policy)
        .with(log);

    if cfg!(feature = "https") {
        warp::serve(routes)
            .tls()
            .key_path("certificates/privkey.pem")
            .cert_path("certificates/fullchain.pem")
            .run(([0, 0, 0, 0], 3030))
            .await;
    } else {
        warp::serve(routes.clone()).run(([0, 0, 0, 0], 3030)).await;
    }
}
