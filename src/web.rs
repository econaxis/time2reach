use crate::formatter::{get_route_mode, time_to_point};
use crate::road_structure::{EdgeId, RoadStructureInner};
use crate::time_to_reach::Configuration;
use crate::{Gtfs1, gtfs_setup, NULL_ID, RoadStructure, Time, time_to_reach};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::{Filter, Reply};
use crate::gtfs_processing::SpatialStopsWithTrips;
use crate::gtfs_setup::get_agency_id_from_short_name;

fn process_coordinates(ad: &mut AppData, lat: f64, lng: f64, include_agencies: Vec<String>) -> impl Reply {
    let gtfs = &ad.gtfs;
    let spatial_stops = &ad.spatial;
    let rs_template = ad.rs_template.clone();
    let rs = RoadStructure::new_from_road_structure(rs_template);

    ad.rs_list.push(rs);
    let mut rs = ad.rs_list.last_mut().unwrap();

    let agency_ids: HashSet<u8> = include_agencies.iter().map(|ag| get_agency_id_from_short_name(ag)).collect();

    let _answer = time_to_reach::generate_reach_times(
        gtfs,
        spatial_stops,
        &mut rs,
        Configuration {
            start_time: Time(17.3 * 3600.0),
            duration_secs: 3600.0 * 1.5,
            location: LatLng {
                latitude: lat,
                longitude: lng,
            },
            agency_ids,
        },
    );

    let edge_times = rs.save();
    let edge_times_object: HashMap<EdgeId, u32> = edge_times
        .into_iter()
        .map(|edge_time| (edge_time.edge_id, edge_time.time as u32))
        .collect();
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
    rs_list: Vec<RoadStructure>,
}

impl AppData {
    fn new(gtfs: Gtfs1, spatial: SpatialStopsWithTrips) -> Arc<Mutex<AppData>> {
        Arc::new(Mutex::new(Self::new1(gtfs, spatial)))
    }
    fn new1(gtfs: Gtfs1, spatial: SpatialStopsWithTrips) -> AppData {
        let rs = RoadStructureInner::new();
        AppData {
            gtfs,
            spatial,
            rs_template: Arc::new(rs),
            rs_list: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct CalculateRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub agencies: Vec<String>,
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
struct TripDetailsInner {
    time: f64,
    line: String,
    stop: String,
}

#[derive(Serialize, Deserialize)]
struct TripDetails {
    background_color: String,
    text_color: String,
    mode: &'static str,
    boarding: TripDetailsInner,
    exit: TripDetailsInner,
}

fn get_trip_details(ad: &mut AppData, id: usize, latlng: LatLng) -> impl Reply {
    if id >= ad.rs_list.len() {
        return warp::reply::json(&"Invalid");
    }
    let rs = &ad.rs_list[id];
    let formatter = time_to_point(
        rs,
        &rs.trips_arena,
        &ad.gtfs,
        [latlng.latitude, latlng.longitude],
        true,
    );

    if formatter.is_none() {
        return warp::reply::json(&"None");
    }

    let mut details_list = Vec::new();

    // Automatically skips
    let mut has_free_transfer_from_prev = false;
    for trip in formatter.unwrap().trips {

        if trip.current_route.route_id == NULL_ID {
            // Begin of trip. Skip here.
            continue;
        }

        let route = &ad.gtfs.routes[&trip.current_route.route_id];
        let boarding_stop = &ad.gtfs.stops[&trip.boarding_stop_id];
        let exit_stop = &ad.gtfs.stops[&trip.get_off_stop_id];

        let exit_stop_msg = if has_free_transfer_from_prev {
            format!("{} (stay on vehicle)", exit_stop.name)
        } else {
            exit_stop.name.clone()
        };
        details_list.push(TripDetails {
            mode: get_route_mode(&ad.gtfs, trip),
            background_color: route.color.clone(),
            text_color: route.text_color.clone(),
            boarding: TripDetailsInner {
                time: trip.boarding_time.0,
                line: route.short_name.clone(),
                stop: boarding_stop.name.clone()
            },
            exit: TripDetailsInner {
                time: trip.exit_time.0,
                line: route.short_name.clone(),
                stop: exit_stop_msg
            }
        });

        has_free_transfer_from_prev = trip.is_free_transfer;
    }


    details_list.reverse();
    warp::reply::json(&details_list)
}

fn with_appdata(
    ad: Arc<Mutex<AppData>>,
) -> impl Filter<Extract=(Arc<Mutex<AppData>>, ), Error=Infallible> + Clone {
    warp::any().map(move || ad.clone())
}

pub async fn main() {
    info!("Loading...");

    let gtfs = crate::setup_gtfs();
    let data = gtfs_setup::generate_stops_trips(&gtfs).to_spatial(&gtfs);

    let appdata = AppData::new(gtfs, data);

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
        .and(warp::path("hello"))
        .and(warp::body::json())
        .map(|ad: Arc<Mutex<AppData>>, req: CalculateRequest| {
            let mut ad = ad.lock().unwrap();
            process_coordinates(&mut ad, req.latitude, req.longitude, req.agencies)
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

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
