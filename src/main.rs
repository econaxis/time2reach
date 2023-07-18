#![feature(trivial_bounds)]
#![feature(slice_group_by)]
#![feature(file_create_new)]
#![feature(vec_into_raw_parts)]

mod agencies;
mod best_times;
mod configuration;
mod formatter;
mod gtfs_processing;
mod gtfs_setup;
mod in_progress_trip;
mod path_usage;
mod projection;
mod reach_data;
mod road_structure;
mod serialization;
mod time;
mod time_to_reach;
mod trip_details;
mod trips_arena;
mod web;
mod web_app_data;
mod web_cache;

#[macro_use]
pub(crate) mod cache_function;

use gtfs_structure_2::gtfs_wrapper::DirectionType;

use rustc_hash::FxHashSet;

use gtfs_structure_2::IdType;
use std::time::Instant;


use crate::road_structure::RoadStructure;
use crate::web::LatLng;
use configuration::Configuration;
use gtfs_structure_2::gtfs_wrapper::Gtfs1;

use crate::agencies::City;
use crate::formatter::time_to_point;
use time::Time;
use trips_arena::TripsArena;

const WALKING_SPEED: f64 = 1.42;
const STRAIGHT_WALKING_SPEED: f64 = 1.25;
pub const MIN_TRANSFER_SECONDS: f64 = 35.0;
pub const TRANSIT_EXIT_PENALTY: f64 = 10.0;
const NULL_ID: (u8, u64) = (u8::MAX, u64::MAX);

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct BusPickupInfo {
    timestamp: Time,
    stop_sequence_no: u16,
    trip_id: IdType,
}

fn direction_to_bool(d: &DirectionType) -> bool {
    match d {
        DirectionType::Outbound => true,
        DirectionType::Inbound => false,
    }
}

fn main1() {
    let gtfs = setup_gtfs();
    let data = gtfs_setup::generate_stops_trips(&gtfs).into_spatial(&City::Toronto, &gtfs);

    let mut rs = RoadStructure::new_toronto();
    let time = Instant::now();
    for _ in 0..40 {
        rs.clear_data();
        time_to_reach::generate_reach_times(
            &gtfs,
            &data,
            &mut rs,
            Configuration {
                // start_time: Time(3600.0 * 13.0),
                start_time: Time(3600.0 * 17.0 + 60.0 * 20.0),
                duration_secs: 3600.0 * 2.0,
                location: LatLng::from_lat_lng(43.64466712433209, -79.38041754904549),
                agency_ids: FxHashSet::from_iter([gtfs.agency_id]),
                modes: vec![],
            },
        );
        let _fmter = time_to_point(
            &rs,
            &rs.trips_arena,
            &gtfs,
            [43.71675866093244, -79.74566916652475],
            true,
        );
        // println!("{}", fmter.unwrap());
    }
    println!("Elapsed: {}", time.elapsed().as_secs_f32());
}

fn main() {
    env_logger::builder()
        .parse_filters("debug")
        .parse_default_env()
        .init();

    if false {
        main1();
    } else {
        // let rt = runtime::Builder::new_multi_thread()
        //     .worker_threads(4)
        //     .build()
        //     .unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            web::main().await;
        });
    }
}

fn setup_gtfs() -> Gtfs1 {
    let mut result = agencies::load_all_gtfs();
    result.remove(&City::Toronto).unwrap()
}
