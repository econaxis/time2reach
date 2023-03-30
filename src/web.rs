use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use id_arena::{Arena, Id};
use serde::Deserialize;
use serde_json::json;
use log::info;
use warp::{Filter, Reply};
use gtfs_structures::TimepointType::Approximate;
use crate::{Gtfs1, gtfs_setup, RoadStructure, SpatialStopsWithTrips, Time, time_to_reach};
use crate::formatter::time_to_point;
use crate::road_structure::{EdgeId, RoadStructureInner};
use crate::time_to_reach::Configuration;


fn process_coordinates(ad: &mut AppData, lat: f64, lng: f64) -> impl Reply {
    let gtfs = &ad.gtfs;
    let spatial_stops = &ad.spatial;
    let rs_template = ad.rs_template.clone();
    let mut rs = RoadStructure::new_from_road_structure(rs_template);

    ad.rs_list.push(rs);
    let mut rs = ad.rs_list.last_mut().unwrap();

    let _answer = time_to_reach::generate_reach_times(gtfs, spatial_stops, &mut rs, Configuration {
        start_time: Time(13.0 * 3600.0),
        duration_secs: 3600.0 * 1.5,
        location: LatLng {
            latitude: lat, longitude: lng
        }
    });

    let edge_times = rs.save();
    let edge_times_object: HashMap<EdgeId, u32> = edge_times.into_iter().map(|edge_time| (edge_time.edge_id, edge_time.time as u32)).collect();
    let response = json!({
        "request_id": ad.rs_list.len() - 1,
        "edge_times": edge_times_object
    });
    warp::reply::json(&response)
}

struct AppData {
    gtfs: Gtfs1,
    spatial: SpatialStopsWithTrips,
    rs_template: Arc<RoadStructureInner>,
    rs_list: Vec<RoadStructure>
}

impl AppData {
    fn new(gtfs: Gtfs1, spatial: SpatialStopsWithTrips) -> Arc<Mutex<AppData>> {
        let rs = RoadStructureInner::new();
        Arc::new(Mutex::new(Self::new1(gtfs, spatial)))
    }
    fn new1(gtfs: Gtfs1, spatial: SpatialStopsWithTrips) -> AppData {
        let rs = RoadStructureInner::new();
        AppData {
            gtfs, spatial, rs_template: Arc::new(rs), rs_list: Vec::new()
        }
    }
}


#[derive(Deserialize)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64
}
impl LatLng {
    pub fn from_lat_lng(lat: f64, lng: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lng
        }
    }
}

fn get_trip_details(ad: &mut AppData, id: usize, latlng: LatLng)-> impl Reply {
    if id >= ad.rs_list.len() {
        return warp::reply::json(&"Invalid");
    }
    let rs = &ad.rs_list[id];
    let formatter = time_to_point(rs, &rs.trips_arena, &ad.gtfs, [latlng.latitude, latlng.longitude], true);

    let format_string = formatter.map(|format| format!("{}", format)).unwrap_or("".to_string());
    warp::reply::json(&format_string)
}

fn with_appdata(ad: Arc<Mutex<AppData>>) -> impl Filter<Extract=(Arc<Mutex<AppData>>,), Error=Infallible> + Clone {
    warp::any().map(move || ad.clone())
}
pub async fn main() {
    env_logger::init();
    info!("Loading...");

    let mut gtfs = gtfs_setup::initialize_gtfs_as_bson("/Users/henry.nguyen@snapcommerce.com/Downloads/gtfs");
    gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
        "/Users/henry.nguyen@snapcommerce.com/Downloads/GO_GTFS",
    ));
    gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
        "/Users/henry.nguyen@snapcommerce.com/Downloads/up_express",
    ));
    gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
        "/Users/henry.nguyen@snapcommerce.com/Downloads/yrt",
    ));
    gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
        "/Users/henry.nguyen@snapcommerce.com/Downloads/brampton",
    ));
    gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
        "/Users/henry.nguyen@snapcommerce.com/Downloads/miway",
    ));
    let data = gtfs_setup::generate_stops_trips(&gtfs).to_spatial(&gtfs);

    let appdata = AppData::new(gtfs, data);

    let cors_policy = warp::cors().allow_any_origin()
        .allow_headers(vec!["Access-Control-Allow-Origin", "Origin", "Accept", "X-Requested-With", "Content-Type"])
        .allow_methods(["POST", "GET"]);

    info!("Setup done");

    let log = warp::log("warp");
    let hello = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path("hello"))
        .and(warp::body::json())
        .map(|ad: Arc<Mutex<AppData>>, latlng: LatLng| {
            let mut ad = ad.lock().unwrap();
            process_coordinates(&mut ad,latlng.latitude, latlng.longitude)
        });

    let details = warp::post()
        .and(with_appdata(appdata.clone()))
        .and(warp::path!("details" / usize))
        .and(warp::body::json())
        .map(|ad: Arc<Mutex<AppData>>, id: usize, latlng: LatLng| {
            let mut ad = ad.lock().unwrap();
            get_trip_details(&mut ad, id, latlng)
        });

    let routes = hello.or(details).with(cors_policy).with(log);


    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}