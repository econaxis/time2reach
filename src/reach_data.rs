use crate::in_progress_trip::InProgressTrip;
use crate::time::Time;
use id_arena::Id;

#[derive(PartialOrd, PartialEq, Debug, Clone)]
pub struct ReachData {
    pub timestamp: Time,
    pub progress_trip_id: Option<Id<InProgressTrip>>,
    pub transfers: u8,
    pub walking_length: f64,
}

impl ReachData {
    pub fn with_time_and_dist(&self, time: Time, additional_walking_dist: f64) -> Self {
        ReachData {
            timestamp: time,
            progress_trip_id: self.progress_trip_id,
            transfers: self.transfers,
            walking_length: self.walking_length + additional_walking_dist,
        }
    }
}
