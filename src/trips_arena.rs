use crate::time::Time;
use crate::{BusPickupInfo, IdType, InProgressTrip};
use id_arena::{Arena, Id};
use std::collections::{HashMap, VecDeque};

//
// #[derive(Hash, Debug, PartialEq, Eq)]
// struct TripsAlreadyTakenCache {
//     trip_id
// }
#[derive(Debug, Default)]
pub struct TripsArena {
    explore_queue: VecDeque<id_arena::Id<InProgressTrip>>,
    // TripID -> stop sequence number of boarding
    trips_already_taken: HashMap<IdType, u16>,

    // StopID -> Earliest Arrival time
    stop_arrival_times: HashMap<IdType, Time>,
    arena: Arena<InProgressTrip>,
}

impl TripsArena {
    pub fn should_explore(&mut self, bu: &BusPickupInfo) -> bool {
        if let Some(sequence_no) = self.trips_already_taken.get(&bu.trip_id) {
            // Don't get on this trip if we have already boarded on an earlier stop
            if sequence_no <= &bu.stop_sequence_no {
                return false;
            }
        }
        self.trips_already_taken
            .insert(bu.trip_id, bu.stop_sequence_no);
        true
    }
    pub(crate) fn add_to_explore(&mut self, item: InProgressTrip) -> Option<Id<InProgressTrip>> {
        if let Some(arrival_time) = self.stop_arrival_times.get_mut(&item.get_off_stop_id) {
            if *arrival_time <= item.exit_time {
                // Someone arrived at this stop before us. Don't explore further.
                return None;
            } else {
                // We arrived at the stop before them. Set our best time instead
                *arrival_time = item.exit_time;
            }
        } else {
            self.stop_arrival_times
                .insert(item.get_off_stop_id, item.exit_time);
        }
        let id = self.arena.alloc(item);
        self.explore_queue.push_back(id);
        Some(id)
    }

    pub(crate) fn get_by_id(&self, id: Id<InProgressTrip>) -> &InProgressTrip {
        &self.arena[id]
    }
    pub(crate) fn pop_front(&mut self) -> Option<(InProgressTrip, Id<InProgressTrip>)> {
        let id = self.explore_queue.pop_front()?;
        Some((self.get_by_id(id).clone(), id))
    }
}
