use crate::in_progress_trip::InProgressTrip;
use crate::time::Time;
use crate::{BusPickupInfo, IdType};
use id_arena::{Arena, Id};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use rustc_hash::FxHashMap;

#[derive(PartialEq, Eq, Debug)]
struct HeapIdTrip {
    inner: Id<InProgressTrip>,
    compare: Time,
}

impl PartialOrd for HeapIdTrip {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare.partial_cmp(&other.compare)?.reverse())
    }
}
impl Ord for HeapIdTrip {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare.cmp(&other.compare).reverse()
    }
}
#[derive(Debug, Default)]
pub struct TripsArena {
    explore_queue: BinaryHeap<HeapIdTrip>,
    // TripID -> stop sequence number of boarding
    trips_already_taken: FxHashMap<IdType, u16>,

    // StopID -> Earliest Arrival time
    stop_arrival_times: FxHashMap<IdType, Time>,
    arena: Arena<InProgressTrip>,
}

impl TripsArena {
    pub fn should_explore(&mut self, bu: &BusPickupInfo) -> bool {
        if let Some(sequence_no) = self.trips_already_taken.get(&bu.trip_id) {
            // Don't get on this trip if we have already boarded on an earlier stop
            if sequence_no <= &bu.stop_sequence_no {
                return false;
            }
        } else {
            self.trips_already_taken
                .insert(bu.trip_id, bu.stop_sequence_no);
        }
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
        let item_exit_time = item.exit_time;
        let id = self.arena.alloc(item);
        let heap_struct = HeapIdTrip {
            compare: item_exit_time,
            inner: id,
        };
        self.explore_queue.push(heap_struct);
        Some(id)
    }

    pub(crate) fn get_by_id(&self, id: Id<InProgressTrip>) -> &InProgressTrip {
        &self.arena[id]
    }
    pub(crate) fn pop_front(&mut self) -> Option<(InProgressTrip, Id<InProgressTrip>)> {
        let heap_item = self.explore_queue.pop()?;
        let id = heap_item.inner;

        Some((self.get_by_id(id).clone(), id))
    }
}
