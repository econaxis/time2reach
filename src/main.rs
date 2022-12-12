#![feature(file_create_new)]
#![feature(vec_into_raw_parts)]

mod formatter;
mod gtfs_setup;
mod gtfs_wrapper;
mod projection;
mod serialization;
mod time_to_reach;
mod trips_arena;

use gtfs_structures::{DirectionType, Stop};
use id_arena::{Arena, Id};

use proj::Proj;
use rstar::primitives::GeomWithData;
use rstar::{PointDistance, RTree};
use serde::Serialize;

use bson::to_bson;
use formatter::InProgressTripsFormatter;
use serde_bytes::ByteBuf;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use gdal::Dataset;
use gdal::raster::{Buffer, ColorInterpretation};
use gdal::spatial_ref::SpatialRef;
pub use time_to_reach::TimeToReachRTree;

use crate::gtfs_wrapper::{Gtfs0, Gtfs1, StopTime, Trip};
use gtfs_wrapper::LibraryGTFS;
use serialization::{MapSerialize, TimeGrid};
use trips_arena::TripsArena;
use crate::projection::ZERO_LATLNG;

const WALKING_SPEED: f64 = 1.3;
type IdType = u64;

#[derive(Default, Debug)]
pub struct RoutePickupTimes(HashMap<RouteStopSequence, BTreeSet<BusPickupInfo>>);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Default)]
pub struct RouteStopSequence {
    route_id: IdType,
    direction: bool,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct BusPickupInfo {
    timestamp: u32,
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
            timestamp: stop_time.arrival_time.unwrap(),
            stop_sequence_no: stop_time.stop_sequence,
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
struct Pickup {
    route_id: IdType,
    time: u32,
}
#[derive(Debug)]
struct StopsData<'a> {
    stop: &'a Stop,
    trips_with_time: RoutePickupTimes,
}

#[derive(Debug)]
struct SpatialStopsWithTrips<'a>(rstar::RTree<GeomWithData<[f64; 2], StopsData<'a>>>);

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
    #[inline(never)]
    fn to_spatial(self, gtfs: &Gtfs1) -> SpatialStopsWithTrips<'_> {
        let mut points_data = Vec::new();

        for (stop_id, trips_with_time) in self.0 {
            let stop = &gtfs.stops[&stop_id];
            let stop_coords = projection::project_stop(stop);

            let stops_data = StopsData {
                stop,
                trips_with_time,
            };
            points_data.push(GeomWithData::new(stop_coords, stops_data));
        }

        SpatialStopsWithTrips(RTree::bulk_load(points_data))
    }
}

#[derive(Debug, Clone)]
pub struct InProgressTrip {
    boarding_time: u32,
    exit_time: u32,
    point: [f64; 2],
    current_route: RouteStopSequence,
    total_transfers: u8,
    get_off_stop_id: IdType,
    boarding_stop_id: IdType,
    previous_transfer: Option<Id<InProgressTrip>>,
}

#[derive(PartialOrd, PartialEq, Debug, Serialize)]
struct ReachData {
    timestamp: u32,

    #[serde(skip)]
    progress_trip_id: Id<InProgressTrip>,
    transfers: u8,
}

#[inline(never)]
fn all_stops_along_trip(
    gtfs: &Gtfs1,
    trip_id: IdType,
    start_sequence_no: u16,
    route_info: &RouteStopSequence,
    previous_transfer: Id<InProgressTrip>,
    transfers_remaining: u8,
    explore_queue: &mut TripsArena,
) {
    let stop_times = &gtfs.trips[&trip_id].stop_times;
    let boarding_stop = &stop_times[start_sequence_no as usize - 1];

    for (_stops_travelled, st) in stop_times[start_sequence_no as usize..].iter().enumerate() {
        let point = projection::project_stop(&gtfs.stops[&st.stop_id]);
        let timestamp = st.arrival_time.unwrap();

        explore_queue.add_to_explore(InProgressTrip {
                boarding_time: boarding_stop.arrival_time.unwrap(),
                exit_time: timestamp,
                point,
                current_route: route_info.clone(),
                get_off_stop_id: st.stop_id,
                boarding_stop_id: boarding_stop.stop_id,
                total_transfers: transfers_remaining,
                previous_transfer: Some(previous_transfer),
            });
    }
}

fn explore_from_point(
    gtfs: &Gtfs1,
    data: &SpatialStopsWithTrips,
    ip: InProgressTrip,
    ip_id: Id<InProgressTrip>,
    explore_queue: &mut TripsArena,
) {


    let mut routes_already_taken = HashSet::from([ip.current_route.clone()]);

    for (stop, distance) in data.0.nearest_neighbor_iter_with_distance_2(&ip.point) {
        if distance > 600.0 * 600.0 {
            // Exceeds the walking threshold.
            break;
        }

        let stop_d = &stop.data;

        let time_to_stop = (distance.sqrt() / WALKING_SPEED) as u32;
        const MIN_TRANSFER_SECONDS: u32 = 15;
        let this_timestamp = ip.exit_time + time_to_stop + MIN_TRANSFER_SECONDS;
        for (route_info, route_pickup) in stop_d.trips_with_time.0.iter() {
            // Search for route pickup on or after the starting_timestamp
            if routes_already_taken.contains(route_info) {
                continue;
            }
            let starting_buspickup = BusPickupInfo {
                timestamp: this_timestamp,
                stop_sequence_no: 0,
                trip_id: 0,
            };
            if let Some(next_bus) = route_pickup.range(starting_buspickup..).next() {
                // For all next stops on the line...push into explore_queue to force a transfer
                assert!(next_bus.timestamp >= this_timestamp);

                if explore_queue.should_explore(next_bus) {
                    all_stops_along_trip(
                        gtfs,
                        next_bus.trip_id,
                        next_bus.stop_sequence_no,
                        route_info,
                        ip_id,
                        ip.total_transfers + 1,
                        explore_queue,
                    );

                    routes_already_taken.insert(route_info.clone());
                }
            }
        }
    }
}

#[inline(never)]
fn is_first_reacher(answer: &TimeToReachRTree, point: &[f64; 2], this_timestamp: u32) -> bool {
    for already_reached in answer.tree.locate_within_distance(*point, 75.0 * 75.0) {
        let time_to_walk_there = (already_reached.distance_2(point).sqrt() / WALKING_SPEED) as u32;
        if already_reached.data.timestamp + time_to_walk_there <= this_timestamp {
            return false;
        }
    }
    true
}

struct GTiffOutput {
    dataset: Dataset
}

impl GTiffOutput {
    fn new(path: &str, size_x: usize, size_y: usize) -> Self {

        let driver = gdal::DriverManager::get_driver_by_name("GTiff").unwrap();
        let mut dataset = driver.create_with_band_type::<i32, _>(path, size_x as isize, size_y as isize, 1).unwrap();

        let spatial_ref = SpatialRef::from_proj4(&format!("+proj=merc +lon_0={} +lat_0={} +lat_ts={}", ZERO_LATLNG[1], ZERO_LATLNG[0], ZERO_LATLNG[0])).unwrap();
        let proj = spatial_ref.to_wkt().unwrap();
        dbg!(&proj);

        dataset.set_spatial_ref(&spatial_ref).unwrap();
        dataset.set_projection(&proj).unwrap();

        Self {
            dataset
        }
    }

    fn write_to_raster(&mut self, tg: &TimeGrid) {
        let mut geotransform = [0.0; 6];
        let start = tg.start_coord;
        let end = tg.end_coord;

        geotransform[0] = start[0];
        geotransform[3] = start[1];

        geotransform[1] = tg.calculate_x_scale();
        geotransform[5] = tg.calculate_y_scale();

        self.dataset.set_geo_transform(&geotransform);

        let mut rb = self.dataset.rasterband(1).unwrap();
        rb.set_no_data_value(Some(-1.0)).unwrap();

        let buffer = Buffer {
            size: (tg.x_samples, tg.y_samples),
            data: tg.map.clone()
        };
        rb.write((0, 0), (tg.x_samples, tg.y_samples),&buffer );
        rb.set_color_interpretation(ColorInterpretation::RedBand).unwrap();
        self.dataset.flush_cache();
    }
}

fn main() {
    const MAP_RESOLUTION: usize = 3000;
    let mut gt = GTiffOutput::new("fd1sa", MAP_RESOLUTION, MAP_RESOLUTION);


    let gtfs = gtfs_setup::initialize_gtfs_as_bson("/Users/henry/Downloads/gtfs-2");
    let data = gtfs_setup::generate_stops_trips(&gtfs).to_spatial(&gtfs);

    let mut trips_arena = TripsArena::default();
    trips_arena.add_to_explore(InProgressTrip {
        boarding_time: 9 * 3600,
        exit_time: 9 * 3600,
        point: projection::project_lng_lat(-79.466791, 43.653210),
        current_route: RouteStopSequence::default(),
        get_off_stop_id: 0,
        total_transfers: 0,
        previous_transfer: None,
        boarding_stop_id: 0,
    });

    let mut answer = TimeToReachRTree::default();

    let mut skip_times = 0;
    while let Some((item, id)) = trips_arena.pop_front() {
        if !is_first_reacher(&mut answer, &item.point, item.exit_time) {
            skip_times += 1;
            continue;
        }
        if item.exit_time > (12.0 * 3600.0) as u32 {
            continue;
        }
        if item.total_transfers >= 5 {
            continue;
        }
        answer.add_observation(
            item.point,
            ReachData {
                timestamp: item.exit_time,
                progress_trip_id: id,
                transfers: item.total_transfers,
            },
        );

        explore_from_point(&gtfs, &data, item, id, &mut trips_arena);
    }

    dbg!(answer.tree.size());
    dbg!(skip_times);

    let mut tg = TimeGrid::new(&answer, MAP_RESOLUTION, MAP_RESOLUTION);
    tg.process(&answer, [9 * 3600, 12 * 3600]);
    gt.write_to_raster(&tg);
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

