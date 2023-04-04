#![feature(file_create_new)]
#![feature(vec_into_raw_parts)]

mod best_times;
mod formatter;
mod gtfs_setup;
mod gtfs_wrapper;
mod projection;
mod road_structure;
mod serialization;
mod time;
mod time_to_reach;
mod trips_arena;
mod web;

use crate::gtfs_wrapper::DirectionType;
use id_arena::Id;

use rstar::primitives::GeomWithData;
use rstar::RTree;
use serde::Serialize;





use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use std::time::Instant;
pub use time_to_reach::TimeToReachRTree;

use crate::formatter::time_to_point;
use crate::gtfs_wrapper::{Gtfs0, Gtfs1, StopTime, Trip};
use crate::projection::{project_lng_lat, PROJSTRING};
use crate::road_structure::RoadStructure;
use crate::time_to_reach::Configuration;
use crate::web::LatLng;
use gtfs_wrapper::LibraryGTFS;

use time::Time;
use trips_arena::TripsArena;

const WALKING_SPEED: f64 = 1.35;
const STRAIGHT_WALKING_SPEED: f64 = 0.95;
type IdType = (u8, u64);
const NULL_ID: (u8, u64) = (u8::MAX, u64::MAX);

#[derive(Default, Debug)]
pub struct RoutePickupTimes(HashMap<RouteStopSequence, BTreeSet<BusPickupInfo>>);

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct RouteStopSequence {
    route_id: IdType,
    direction: bool,
}

impl Default for RouteStopSequence {
    fn default() -> Self {
        Self {
            route_id: NULL_ID,
            direction: false,
        }
    }
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

impl RoutePickupTimes {
    fn add_route_pickup_time(&mut self, trip: &Trip, stop_time: &StopTime) {
        let route_stop_sequence = RouteStopSequence {
            route_id: trip.route_id,
            direction: direction_to_bool(&trip.direction_id.unwrap()),
        };

        let bus_pickup = BusPickupInfo {
            timestamp: Time(stop_time.arrival_time.unwrap() as f64),
            stop_sequence_no: stop_time.stop_sequence as u16,
            trip_id: trip.id,
        };
        if let Some(times) = self.0.get_mut(&route_stop_sequence) {
            times.insert(bus_pickup);
        } else {
            self.0
                .insert(route_stop_sequence, BTreeSet::from([bus_pickup]));
        }
    }
}
#[derive(Default)]
pub struct StopsWithTrips(pub HashMap<IdType, RoutePickupTimes>);

#[derive(Debug)]
struct StopsData {
    trips_with_time: RoutePickupTimes,
}

#[derive(Debug)]
pub struct SpatialStopsWithTrips(RTree<GeomWithData<[f64; 2], StopsData>>);

impl StopsWithTrips {
    fn add_stop(&mut self, stop_time: &StopTime, trip: &Trip) {
        if let Some(trips) = self.0.get_mut(&stop_time.stop_id) {
            trips.add_route_pickup_time(trip, stop_time);
        } else {
            let mut rp = RoutePickupTimes::default();
            rp.add_route_pickup_time(trip, stop_time);
            self.0.insert(stop_time.stop_id, rp);
        }
    }
    fn to_spatial(self, gtfs: &Gtfs1) -> SpatialStopsWithTrips {
        let mut points_data = Vec::new();

        for (stop_id, trips_with_time) in self.0 {
            let stop = &gtfs.stops[&stop_id];
            let stop_coords = projection::project_stop(stop);

            let stops_data = StopsData { trips_with_time };
            points_data.push(GeomWithData::new(stop_coords, stops_data));
        }

        SpatialStopsWithTrips(RTree::bulk_load(points_data))
    }
}

#[derive(Debug, Clone)]
pub struct InProgressTrip {
    boarding_time: Time,
    exit_time: Time,
    point: [f64; 2],
    current_route: RouteStopSequence,
    total_transfers: u8,
    get_off_stop_id: IdType,
    boarding_stop_id: IdType,
    previous_transfer: Option<Id<InProgressTrip>>,
}

#[derive(PartialOrd, PartialEq, Eq, Debug, Serialize, Clone)]
pub struct ReachData {
    timestamp: Time,

    #[serde(skip)]
    progress_trip_id: Option<Id<InProgressTrip>>,
    transfers: u8,
}

impl ReachData {
    pub fn new_with_time(time: Time) -> Self {
        Self {
            timestamp: time,
            progress_trip_id: None,
            transfers: 0,
        }
    }
    pub fn with_time(&self, time: Time) -> Self {
        ReachData {
            timestamp: time,
            progress_trip_id: self.progress_trip_id,
            transfers: self.transfers,
        }
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
                agency_ids: HashSet::new()
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
    let mut gtfs =
        gtfs_setup::initialize_gtfs_as_bson(
        "/Users/henry.nguyen@snapcommerce.com/Downloads/up_express", "UP"
        );
    gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
            "/Users/henry.nguyen@snapcommerce.com/Downloads/gtfs", "TTC"
    ));
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry.nguyen@snapcommerce.com/Downloads/GO_GTFS",
    // ));
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry.nguyen@snapcommerce.com/Downloads/yrt",
    // ));
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry.nguyen@snapcommerce.com/Downloads/brampton",
    // ));
    // gtfs.merge(gtfs_setup::initialize_gtfs_as_bson(
    //     "/Users/henry.nguyen@snapcommerce.com/Downloads/miway",
    // ));
    gtfs
}
