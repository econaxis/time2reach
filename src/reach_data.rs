use id_arena::Id;
use crate::in_progress_trip::InProgressTrip;
use crate::time::Time;

#[derive(PartialOrd, PartialEq, Eq, Debug, Clone)]
pub struct ReachData {
    pub timestamp: Time,
    pub progress_trip_id: Option<Id<InProgressTrip>>,
    pub transfers: u8,
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
