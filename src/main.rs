#![feature(trivial_bounds)]
#![feature(file_create_new)]
#![feature(vec_into_raw_parts)]

extern crate core;

mod agencies;
mod best_times;
mod calendar;
mod configuration;
mod formatter;
mod gtfs_processing;
mod gtfs_setup;
mod gtfs_wrapper;
mod in_progress_trip;
mod projection;
mod reach_data;
mod road_structure;
mod serialization;
mod time;
mod time_to_reach;
mod trips_arena;
mod web;

use crate::gtfs_wrapper::DirectionType;

use std::collections::HashSet;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

use crate::formatter::time_to_point;
use crate::gtfs_wrapper::{Gtfs0, Gtfs1};
use crate::projection::PROJSTRING;
use crate::road_structure::RoadStructure;
use crate::web::LatLng;
use configuration::Configuration;
use gtfs_wrapper::LibraryGTFS;

use crate::agencies::City;
use time::Time;
use trips_arena::TripsArena;

const WALKING_SPEED: f64 = 1.42;
const STRAIGHT_WALKING_SPEED: f64 = 1.30;
pub const MIN_TRANSFER_SECONDS: f64 = 5.0;
pub const TRANSIT_EXIT_PENALTY: f64 = 15.0;
type IdType = (u8, u64);
const NULL_ID: (u8, u64) = (u8::MAX, u64::MAX);

lazy_static! {
    pub static ref PRESENT_DAY: NaiveDate = NaiveDate::from_ymd_opt(2023, 04, 04).unwrap();
}

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
    let data = gtfs_setup::generate_stops_trips(&gtfs).to_spatial(&gtfs);

    let mut rs = RoadStructure::new();
    let time = Instant::now();
    for _ in 0..1 {
        rs.clear_data();
        time_to_reach::generate_reach_times(
            &gtfs,
            &data,
            &mut rs,
            Configuration {
                // start_time: Time(3600.0 * 13.0),
                start_time: Time(3600.0 * 13.0),
                duration_secs: 3600.0 * 1.5,
                location: LatLng::from_lat_lng(43.68228522699712, -79.6125297053927),
                agency_ids: HashSet::new(),
                modes: vec![],
            },
        );
        time_to_point(
            &rs,
            &rs.trips_arena,
            &gtfs,
            [43.68208688807143, -79.61316825624802],
            true,
        );
        // rs.save();
    }
    println!("Elapsed: {}", time.elapsed().as_secs_f32());

    // dbg!(answer.tree.size());
    //
    // let mut tg = TimeGrid::new(&answer, MAP_RESOLUTION, MAP_RESOLUTION);
    // tg.process(&answer);
    // gt.write_to_raster(&mut tg);
    // let mut file = File::create("observations.rmp").unwrap();
    // file.write(
    //     &rmp_serde::to_vec_named(&MapSerialize {
    //         map: unsafe { serialization::to_bytebuf(tg.map) },
    //         x: tg.x_samples,
    //         y: tg.y_samples,
    //     })
    //     .unwrap(),
    // )
    // .unwrap();
}

fn main() {
    env_logger::init();

    if false {
        main1();
        return;
    } else {
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
