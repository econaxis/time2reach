use crate::agencies::{load_all_gtfs, Agency, City};

use futures::StreamExt;

use crate::configuration::Configuration;
use crate::gtfs_setup::get_agency_id_from_short_name;
use bike::{route, RouteResponse, RouteOptions};
use crate::road_structure::EdgeId;
use crate::{gtfs_setup, time_to_reach, trip_details, Gtfs1, RoadStructure, Time};
use gtfs_structure_2::gtfs_wrapper::RouteType;

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_json::json;

use std::convert::Infallible;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::PathBuf;
use std::{fmt, io};

use anyhow::{anyhow, Context};
use futures::stream::FuturesUnordered;
use std::sync::Arc;
use geojson::GeoJson;

use crate::trip_details::CalculateRequest;
use crate::web_app_data::{AllAppData, CacheKey, CityAppData};
use crate::web_cache::{check_cache, insert_cache};
use warp::http::HeaderValue;
use warp::hyper::StatusCode;
use warp::log::{Info, Log};
use warp::path::{Exact, Tail};
use warp::reject::Reject;
use warp::reply::Json;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

fn gtfs_to_city_appdata(city: City, gtfs: Gtfs1) -> CityAppData {
    let data = gtfs_setup::generate_stops_trips(&gtfs).into_spatial(&city, &gtfs);

    CityAppData::new1(gtfs, data, city)
}

fn check_city(ad: &Arc<AllAppData>, lat: f64, lng: f64) -> Option<City> {
    for (city, data) in &ad.ads {
        let is_near_point = data.spatial.is_near_point(
            city,
            LatLng {
                latitude: lat,
                longitude: lng,
            },
        );

        if is_near_point {
            return Some(*city);
        }
    }
    None
}

#[derive(Debug)]
struct BadQuery {
    reason: String,
}

impl Reject for BadQuery {}

impl From<&str> for BadQuery {
    fn from(value: &str) -> Self {
        BadQuery {
            reason: value.to_string(),
        }
    }
}

fn process_coordinates(
    ad: Arc<AllAppData>,
    lat: f64,
    lng: f64,
    include_agencies: Vec<String>,
    include_modes: Vec<String>,
    start_time: u64,
    max_search_time: f64,
    _prev_request_id: Option<RequestId>,
) -> Result<warp::reply::Json, BadQuery> {
    let city = check_city(&ad, lat, lng);

    if city.is_none() {
        return Err(BadQuery::from("Invalid city"));
    }

    if max_search_time >= 3.5 * 3600.0 {
        log::warn!("Invalid max search time");
        return Err(BadQuery::from("Invalid max search time"));
    }
    let city = city.unwrap();
    let ad = &ad.ads.get(&city).unwrap();

    let cache_key = match check_cache(
        ad,
        lat,
        lng,
        &include_agencies,
        &include_modes,
        start_time,
        max_search_time as u64,
    ) {
        Ok(reply) => return Ok(reply),
        Err(key) => key,
    };

    let gtfs = &ad.gtfs;
    let spatial_stops = &ad.spatial;
    let rs_template = ad.rs_template.clone();
    let mut rs = RoadStructure::new_from_road_structure(rs_template);

    let agency_ids: FxHashSet<u16> = include_agencies
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
            duration_secs: max_search_time,
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

    Ok(insert_cache(cache_key, response, request_id))
}

#[derive(Deserialize, Clone, Copy)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

impl From<LatLng> for bike::Point {
    fn from(value: LatLng) -> Self {
        Self {
            lat: value.latitude,
            lon: value.longitude,
            ele: 0.0,
        }
    }
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
    pub rs_list_index: CacheKey,
    pub city: City,
}

fn with_appdata(
    ad: Arc<AllAppData>,
) -> impl Filter<Extract = (Arc<AllAppData>,), Error = Infallible> + Clone {
    warp::any().map(move || ad.clone())
}

struct OptFmt<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for OptFmt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref t) = self.0 {
            fmt::Display::fmt(t, f)
        } else {
            f.write_str("-")
        }
    }
}

pub fn weblog(name: &'static str) -> Log<impl Fn(Info<'_>) + Copy> {
    warp::log::custom(|info: Info<'_>| {
        let cf_connecting = info
            .request_headers()
            .get("Cf-Connecting-Ip")
            .map(|a| a.to_str().unwrap());
        log::info!(
            target: name,
            "{} {} \"{} {} {:?}\" {} \"{}\" \"{}\" {:?}",
            OptFmt(info.remote_addr()),
            OptFmt(cf_connecting),
            info.method(),
            info.path(),
            info.version(),
            info.status().as_u16(),
            OptFmt(info.referer()),
            OptFmt(info.user_agent()),
            info.elapsed(),
        );
    })
}

fn get_file(str: Tail) -> anyhow::Result<Vec<u8>> {
    let str = str.as_str();
    let str = str.trim_end_matches(".bin");
    let parts: Vec<_> = str.split('/').collect();

    if parts.len() != 4 {
        return Err(anyhow!("Invalid path"));
    }

    let str = parts[0];

    if str != "all_cities" {
        return Err(anyhow!("Invalid path"));
    }

    println!("Path: {:?}", parts);
    let z: u32 = parts[1].parse()?;
    let x: u32 = parts[2].parse()?;
    let y: u32 = parts[3].parse()?;

    let mut path = PathBuf::from("vancouver-cache");
    path.push(format!("{}/{}/{}/{}.pbf", str, z, x, y));
    // Read file from path
    println!("Trying to read at {path:?}");
    let mut file = File::open(path)?;
    let mut answer = Vec::new();
    file.read_to_end(&mut answer)?;
    Ok(answer)
}

#[derive(Serialize, Deserialize)]
struct IDQuery {
    id: Option<u64>,
}

#[derive(Deserialize)]
struct BikeCalculateRequest {
    start: LatLng,
    end: LatLng,
    options: Option<RouteOptions>
}
pub fn bike_endpoints(appdata: Arc<AllAppData>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let bike_endpoint = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path!("bike"))
        .and(warp::body::json())
        .map(|ad: Arc<AllAppData>, req: serde_json::Value | {
            let req: BikeCalculateRequest = serde_json::from_value(req).with_context(|| "JSON Deserialization Error")?;

            route(
                &ad.bikegraph,
                req.start.into(),
                req.end.into(),
                req.options.unwrap_or_default()
            )
        })
        .map(|r: anyhow::Result<RouteResponse>| match r {
            Ok(a) => warp::reply::json(&a).into_response(),
            Err(e) => warp::reply::with_status(format!("{:#}", e), StatusCode::BAD_REQUEST).into_response(),
        });

    bike_endpoint
}
pub async fn main() {
    let all_gtfs = load_all_gtfs();
    let agencies: Vec<Agency> = Vec::new();
    let agencies: Vec<Agency> = all_gtfs.values().map(|a| &a.1).flatten().cloned().collect();
    println!("Agencies is {:?}", agencies);
    let all_gtfs_future = all_gtfs.into_iter().map(|(city, (gtfs, _agency))| {
        tokio::task::spawn_blocking(move || (city, gtfs_to_city_appdata(city, gtfs)))
    });


    let mut all_gtfs: FxHashMap<City, CityAppData> = FxHashMap::default();
    let mut temp = FuturesUnordered::from_iter(all_gtfs_future);
    while let Some(result) = temp.next().await {
        let (city, ad) = result.unwrap();
        all_gtfs.insert(city, ad);
    }
    let appdata = Arc::new(AllAppData { ads: all_gtfs, bikegraph: Arc::new(bike::parse_graph()) });

    let bike_endpoint = bike_endpoints(appdata.clone());

    let cors_policy = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Origin",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
            "content-type",
            "content-length",
            "date"
        ])
        .allow_methods(["POST", "GET"]);

    let log = weblog("warp");
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
                req.max_search_time,
                req.previous_request_id,
            )
        })
        .map(|r: Result<Json, BadQuery>| match r {
            Ok(a) => warp::reply::with_status(a, StatusCode::OK).into_response(),
            Err(e) => warp::reply::with_status(e.reason, StatusCode::BAD_REQUEST).into_response(),
        });

    let details = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path!("details"))
        .and(warp::body::json())
        .map(trip_details::get_trip_details)
        .map(|result: Result<warp::reply::Json, &'static str>| {
            result.map(|a| a.into_response()).unwrap_or_else(|err| {
                warp::reply::with_status(err, StatusCode::INTERNAL_SERVER_ERROR).into_response()
            })
        });

    let agencies_endpoint = warp::get()
        .and(warp::path!("agencies"))
        .and(warp::query::<IDQuery>())
        .map(move |id: IDQuery| {
            log::info!("Requested agency with ID {}", id.id.unwrap_or(0));
            warp::reply::json(&agencies)
        })
        .map(|response: Json| {
            let mut resp = response.into_response();
            let headers = resp.headers_mut();
            headers.append("Cache-Control", HeaderValue::from_static("max-age=18000"));
            resp
        });

    let mvt_endpoint = warp::get()
        .and(warp::path("mvt"))
        .and(warp::path::tail())
        .map(get_file)
        .map(|result: anyhow::Result<Vec<u8>>| match result {
            Ok(a) => warp::reply::with_status(a, StatusCode::OK).into_response(),
            Err(err) => {
                if let Some(ioerr) = err.downcast_ref::<io::Error>() {
                    if ioerr.kind() == ErrorKind::NotFound {
                        return warp::reply::with_status("Not found", StatusCode::NOT_FOUND)
                            .into_response();
                    }
                }
                warp::reply::with_status(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        })
        .map(|mut response: Response| {
            if response.status() == StatusCode::OK {
                let headers = response.headers_mut();
                headers.append("Content-Encoding", HeaderValue::from_static("gzip"));
                headers.append(
                    "Content-Type",
                    HeaderValue::from_static("application/x-protobuf"),
                );
                headers.append("Cache-Control", HeaderValue::from_static("max-age=18000"));
            }
            response
        });

    let routes = agencies_endpoint
        .or(details)
        .or(mvt_endpoint)
        .or(hello)
        .or(bike_endpoint)
        .with(cors_policy)
        .with(log);

    if cfg!(feature = "https") {
        warp::serve(routes)
            .tls()
            .key_path("certificates/cf-privkey.pem")
            .cert_path("certificates/cf-cert.pem")
            .run(([0, 0, 0, 0], 3030))
            .await;
    } else {
        warp::serve(routes.clone()).run(([0, 0, 0, 0], 3030)).await;
    }
}
