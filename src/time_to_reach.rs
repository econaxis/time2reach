use crate::road_structure::RoadStructure;
use crate::{
    projection, BusPickupInfo, Gtfs1, IdType, InProgressTrip, ReachData, RouteStopSequence,
    SpatialStopsWithTrips, TripsArena, NULL_ID, WALKING_SPEED,
};
use id_arena::Id;
use rstar::primitives::GeomWithData;
use rstar::{PointDistance, RTree};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct TimeToReachRTree {
    pub(crate) tree: RTree<GeomWithData<[f64; 2], ReachData>>,
}
pub fn calculate_score(original_point: &[f64; 2], obs: &GeomWithData<[f64; 2], ReachData>) -> u32 {
    let distance = obs.distance_2(&original_point).sqrt();

    let time_to_reach = distance / WALKING_SPEED;
    let mut penalty = 10.0;
    if time_to_reach >= 25.0 {
        penalty += 1.5 * time_to_reach;
    } else {
        penalty += (time_to_reach - 25.0) * 7.0;
    }

    if time_to_reach >= 2.0 * 60.0 {
        penalty += 1.0 * (time_to_reach - 2.0 * 60.0);
    }

    if obs.data.transfers >= 2 {
        // Penalize time for every transfer performed
        penalty += (obs.data.transfers as u32 - 1) as f64 * 20.0
    }

    let mut time_to_reach = time_to_reach + obs.data.timestamp as f64 + penalty;
    let mut time_to_reach = time_to_reach as u32;
    if time_to_reach < obs.data.timestamp {
        time_to_reach = obs.data.timestamp;
    }
    time_to_reach
}

impl TimeToReachRTree {
    fn serialize_to_json(&self) -> Vec<serde_json::Value> {
        self.tree
            .iter()
            .map(|doc| {
                serde_json::json! ({
                    "point": doc.geom().as_slice(),
                    "data": bson::to_bson(&doc.data).unwrap()
                })
            })
            .collect()
    }
    pub(crate) fn add_observation(&mut self, point: [f64; 2], mut data: ReachData) {
        for near in self.tree.drain_within_distance(point, 15.0 * 15.0) {
            let time_to_walk_here =
                (near.distance_2(&point) / WALKING_SPEED) as u32 + near.data.timestamp;
            if time_to_walk_here < data.timestamp {
                data.timestamp = time_to_walk_here;
            }
        }

        self.tree.insert(GeomWithData::new(point, data));
    }

    pub(crate) fn sample_fastest_time(&self, point: [f64; 2]) -> Option<u32> {
        self.sample_fastest_time_within_distance(point, 600.0)
            .or_else(|| self.sample_fastest_time_within_distance(point, 1500.0))
    }

    fn sample_fastest_time_within_distance(&self, point: [f64; 2], distance: f64) -> Option<u32> {
        let best_time1 = self
            .tree
            .locate_within_distance(point, distance * distance)
            .map(|obs| calculate_score(&point, obs))
            .min();

        best_time1
    }
}

pub fn generate_reach_times(
    gtfs: &Gtfs1,
    data: &SpatialStopsWithTrips,
    rs: &mut RoadStructure,
) -> TimeToReachRTree {
    let mut trips_arena = TripsArena::default();
    trips_arena.add_to_explore(InProgressTrip {
        boarding_time: 13 * 3600,
        exit_time: 13 * 3600,
        point: projection::project_lng_lat(-79.450641, 43.657628),
        current_route: RouteStopSequence::default(),
        get_off_stop_id: NULL_ID,
        total_transfers: 0,
        previous_transfer: None,
        boarding_stop_id: NULL_ID,
    });

    let mut answer = TimeToReachRTree::default();

    while let Some((item, id)) = trips_arena.pop_front() {
        if !rs.is_first_reacher(&item.point, item.exit_time) {
            continue;
        }
        if item.exit_time > (13.5 * 3600.0) as u32 {
            continue;
        }
        if item.total_transfers >= 5 {
            continue;
        }

        rs.add_observation(
            &item.point,
            ReachData {
                timestamp: item.exit_time,
                progress_trip_id: id,
                transfers: item.total_transfers,
            },
        );
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
    answer
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
    let boarding_stop = &stop_times[start_sequence_no as usize];

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
                trip_id: NULL_ID,
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
