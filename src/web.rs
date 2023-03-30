use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::Deserialize;

use warp::{Filter, Reply};
use crate::{Gtfs1, gtfs_setup, RoadStructure, SpatialStopsWithTrips, Time, time_to_reach};
use crate::road_structure::{EdgeId, RoadStructureInner};
use crate::time_to_reach::Configuration;


fn process_coordinates(gtfs: &Gtfs1, spatial_stops: &SpatialStopsWithTrips,rs: Arc<RoadStructureInner>, lat: f64, lng: f64) -> impl Reply {
    let mut rs = RoadStructure::new_from_road_structure(rs);

    let _answer = time_to_reach::generate_reach_times(gtfs, spatial_stops, &mut rs, Configuration {
        start_time: Time(13.0 * 3600.0),
        duration_secs: 3600.0 * 1.5,
        location: LatLng {
            latitude: lat, longitude: lng
        }
    });

    let edge_times = rs.save();
    let edge_times_object: HashMap<EdgeId, u32> = edge_times.into_iter().map(|edge_time| (edge_time.edge_id, edge_time.time as u32)).collect();
    warp::reply::json(&edge_times_object)
}

struct AppData {
    gtfs: Gtfs1,
    spatial: SpatialStopsWithTrips,
    rs_template: Arc<RoadStructureInner>
}

impl AppData {
    fn new(gtfs: Gtfs1, spatial: SpatialStopsWithTrips) -> Arc<Mutex<AppData>> {
        let rs = RoadStructureInner::new();
        Arc::new(Mutex::new(AppData {
            gtfs, spatial, rs_template: Arc::new(rs)
        }))
    }
    fn new1(gtfs: Gtfs1, spatial: SpatialStopsWithTrips) -> AppData {
        let rs = RoadStructureInner::new();
        AppData {
            gtfs, spatial, rs_template: Arc::new(rs)
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

pub async fn main() {
    env_logger::init();
    println!("Loading...");
    // let mut gtfs = gtfs_setup::initialize_gtfs_as_bson("/Users/henry/Downloads/UP_Express");
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry/Downloads/GO_GTFS",
    // ));
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry/Downloads/gtfs-2"
    // ));
    let mut gtfs = gtfs_setup::initialize_gtfs_as_bson("/Users/henry.nguyen@snapcommerce.com/Downloads/gtfs");
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry.nguyen@snapcommerce.com/Downloads/GO_GTFS",
    // ));
    let data = gtfs_setup::generate_stops_trips(&gtfs).to_spatial(&gtfs);

    let appdata = AppData::new(gtfs, data);

    let cors_policy = warp::cors().allow_any_origin()
        .allow_headers(vec!["Access-Control-Allow-Origin", "Origin", "Accept", "X-Requested-With", "Content-Type"])
        .allow_methods(["POST", "GET"]);

    println!("Setup done");
    let hello = warp::post()
        .and(warp::path("hello"))
        .and(warp::body::json())
        .map(move |latlng: LatLng| {
            let ad = appdata.clone();
            let ad = ad.lock().unwrap();
            process_coordinates(&ad.gtfs, &ad.spatial, ad.rs_template.clone(),latlng.latitude, latlng.longitude)
        }).with(cors_policy);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}