use crate::configuration::Configuration;
use crate::gtfs_processing::{RouteStopSequence, SpatialStopsWithTrips};
use crate::in_progress_trip::InProgressTrip;
use crate::reach_data::ReachData;
use crate::road_structure::RoadStructure;
use crate::{
    projection, BusPickupInfo, Gtfs1, TripsArena, MIN_TRANSFER_SECONDS, NULL_ID,
    STRAIGHT_WALKING_SPEED, TRANSIT_EXIT_PENALTY,
};
use gtfs_structure_2::gtfs_wrapper::StopTime;
use gtfs_structure_2::IdType;

use crate::agencies::City;
use chrono::Utc;
use id_arena::Id;
use rustc_hash::FxHashSet;

use crate::time::Time;

// #[derive(Debug, Default)]
// pub struct TimeToReachRTree {
//     pub(crate) tree: RTree<GeomWithData<[f64; 2], ReachData>>,
// }
// pub fn calculate_score(original_point: &[f64; 2], obs: &GeomWithData<[f64; 2], ReachData>) -> Time {
//     let distance = obs.distance_2(original_point).sqrt();
//
//     let time_to_reach = distance / WALKING_SPEED;
//     let mut penalty = 10.0;
//     if time_to_reach >= 25.0 {
//         penalty += 1.5 * time_to_reach;
//     } else {
//         penalty += (time_to_reach - 25.0) * 7.0;
//     }
//
//     if time_to_reach >= 2.0 * 60.0 {
//         penalty += 1.0 * (time_to_reach - 2.0 * 60.0);
//     }
//
//     if obs.data.transfers >= 2 {
//         // Penalize time for every transfer performed
//         penalty += (obs.data.transfers as u32 - 1) as f64 * 20.0
//     }
//
//     let mut time_to_reach = obs.data.timestamp + penalty + time_to_reach;
//     if time_to_reach < obs.data.timestamp {
//         time_to_reach = obs.data.timestamp;
//     }
//     time_to_reach
// }
//
// impl TimeToReachRTree {
//     pub(crate) fn add_observation(&mut self, point: [f64; 2], mut data: ReachData) {
//         for near in self.tree.drain_within_distance(point, 15.0 * 15.0) {
//             let time_to_walk_here = near.data.timestamp + near.distance_2(&point) / WALKING_SPEED;
//             if time_to_walk_here < data.timestamp {
//                 data.timestamp = time_to_walk_here;
//             }
//         }
//
//         self.tree.insert(GeomWithData::new(point, data));
//     }
//
//     // pub(crate) fn sample_fastest_time(&self, point: [f64; 2]) -> Option<u32> {
//     //     self.sample_fastest_time_within_distance(point, 600.0)
//     //         .or_else(|| self.sample_fastest_time_within_distance(point, 1500.0))
//     // }
//
//     // fn sample_fastest_time_within_distance(&self, point: [f64; 2], distance: f64) -> Option<u32> {
//     //     let best_time1 = self
//     //         .tree
//     //         .locate_within_distance(point, distance * distance)
//     //         .map(|obs| calculate_score(&point, obs))
//     //         .min();
//     //
//     //     best_time1
//     // }
// }

pub fn generate_reach_times(
    gtfs: &Gtfs1,
    data: &SpatialStopsWithTrips,
    rs: &mut RoadStructure,
    config: Configuration,
) {
    const MAX_TRANSFERS: u8 = 4;
    let location = config.location;
    rs.trips_arena.add_to_explore(InProgressTrip {
        trip_id: NULL_ID,
        boarding_time: config.start_time,
        exit_time: config.start_time,
        point: projection::project_lng_lat(rs.city(), location.longitude, location.latitude),
        current_route: RouteStopSequence::default(),
        get_off_stop_id: NULL_ID,
        total_transfers: 0,
        previous_transfer: None,
        boarding_stop_id: NULL_ID,
        is_free_transfer: false,
        walking_time: Time(0.0),
        walking_length_m: 0.0,
        boarding_stop_time_idx: 0,
        get_off_stop_time_idx: 0,
    }, config.transfer_cost);

    while let Some((item, id)) = rs.trips_arena.pop_front() {
        if item.exit_time > config.start_time + config.duration_secs {
            continue;
        }
        if item.total_transfers > MAX_TRANSFERS {
            continue;
        }
        if !rs.is_first_reacher_to_stop(item.get_off_stop_id, &item.point, item.exit_time) {
            continue;
        }

        rs.add_observation(
            &item.point,
            ReachData {
                timestamp: item.exit_time + TRANSIT_EXIT_PENALTY,
                progress_trip_id: Some(id),
                transfers: item.total_transfers,
                walking_length: 0.0,
            },
        );
        let city = *rs.city();
        explore_from_point(&city, gtfs, data, item, id, &mut rs.trips_arena, &config);
    }
}

fn get_stop_from_stop_seq_no(stop_times: &[StopTime], stop_sequence_no: u16) -> (&StopTime, usize) {
    for i in 0..=stop_sequence_no as usize {
        if stop_times[i].stop_sequence == stop_sequence_no {
            return (&stop_times[i], i);
        }
    }
    unreachable!("{:?} {:?}", stop_times, stop_sequence_no)
}

fn all_stops_along_trip(
    city: &City,
    gtfs: &Gtfs1,
    trip_id: IdType,
    start_sequence_no: u16,
    route_info: &RouteStopSequence,
    previous_transfer_id: Id<InProgressTrip>,
    transfers_remaining: u8,
    explore_queue: &mut TripsArena,
    transfer_walking_time: Time,
    transfer_walking_length: f64,
    transfer_cost: u64
) {
    let is_free_transfer = explore_queue
        .get_by_id(previous_transfer_id)
        .is_free_transfer;
    let stop_times = &gtfs.trips[&trip_id].stop_times;
    let (boarding_stop, stop_time_index) = get_stop_from_stop_seq_no(stop_times, start_sequence_no);

    for (_stops_travelled, st) in stop_times[stop_time_index + 1..].iter().enumerate() {
        let stop = &gtfs.stops[&st.stop_id];
        let point = projection::project_stop(city, stop);
        if st.arrival_time.is_none() {
            println!("Stop time arrival is none {}", st.stop_id.0);
        }

        let timestamp = st.arrival_time.unwrap();

        let current_inprogress_trip = InProgressTrip {
            trip_id,
            boarding_time: Time(boarding_stop.arrival_time.unwrap() as f64),
            exit_time: Time(timestamp as f64),
            point,
            current_route: route_info.clone(),
            get_off_stop_id: st.stop_id,
            boarding_stop_id: boarding_stop.stop_id,
            total_transfers: transfers_remaining,
            previous_transfer: Some(previous_transfer_id),
            is_free_transfer,
            walking_time: transfer_walking_time,
            walking_length_m: transfer_walking_length as f32,
            boarding_stop_time_idx: boarding_stop.index_of_stop_time,
            get_off_stop_time_idx: st.index_of_stop_time,
        };

        let id = explore_queue.add_to_explore(current_inprogress_trip, transfer_cost);

        if id.is_none() {
            break;
        }
    }
}

fn explore_from_point(
    city: &City,
    gtfs: &Gtfs1,
    data: &SpatialStopsWithTrips,
    ip: InProgressTrip,
    ip_id: Id<InProgressTrip>,
    explore_queue: &mut TripsArena,
    config: &Configuration,
) {
    let today = Utc::now().date_naive();

    let mut routes_already_taken = FxHashSet::from_iter([ip.current_route.clone()]);
    let current_trip = &gtfs.trips.get(&ip.trip_id);

    for (stop, distance) in data.0.nearest_neighbor_iter_with_distance_2(&ip.point) {
        if distance > 800.0 * 800.0 {
            // Exceeds the walking threshold.
            break;
        }

        let stop_d = &stop.data;

        let transfer_walking_length = distance.sqrt();
        let time_to_stop = transfer_walking_length / STRAIGHT_WALKING_SPEED;
        let this_timestamp = ip.exit_time + time_to_stop + MIN_TRANSFER_SECONDS;

        // Search for route pickup on or after the starting_timestamp
        for (route_info, route_pickup) in stop_d.trips_with_time.0.iter() {
            if routes_already_taken.contains(route_info) {
                continue;
            }

            let is_valid_agency = config.agency_ids.contains(&route_info.route_id.0);

            let this_route = &gtfs.routes[&route_info.route_id];

            if !is_valid_agency {
                continue;
            }

            if !(config.modes.is_empty() || config.modes.contains(&this_route.route_type)) {
                continue;
            }

            let starting_buspickup = BusPickupInfo {
                timestamp: this_timestamp,
                stop_sequence_no: 0,
                trip_id: NULL_ID,
            };

            for next_bus in route_pickup.range(starting_buspickup..) {
                debug_assert!(next_bus.timestamp >= this_timestamp);

                let this_trip = &gtfs.trips[&next_bus.trip_id];

                // If the service runs today
                if !gtfs.calendar.runs_on_date(this_trip.service_id, today) {
                    continue;
                }

                let is_free_tranfer = this_trip.block_id.is_some()
                    && this_trip.block_id.as_ref()
                        == current_trip.and_then(|a| a.block_id.as_ref());

                let transfers_remaining = if is_free_tranfer {
                    ip.total_transfers
                } else {
                    ip.total_transfers + 1
                };

                if explore_queue.should_explore(next_bus) {
                    all_stops_along_trip(
                        city,
                        gtfs,
                        next_bus.trip_id,
                        next_bus.stop_sequence_no,
                        route_info,
                        ip_id,
                        transfers_remaining,
                        explore_queue,
                        Time(time_to_stop),
                        transfer_walking_length,
                        config.transfer_cost
                    );
                    routes_already_taken.insert(route_info.clone());
                    break;
                }
            }
        }
    }
}
