use id_arena::Id;
use crate::IdType;
use crate::gtfs_processing::RouteStopSequence;
use crate::time::Time;

#[derive(Debug, Clone)]
pub struct InProgressTrip {
    pub trip_id: IdType,
    pub boarding_time: Time,
    pub exit_time: Time,
    pub point: [f64; 2],
    pub current_route: RouteStopSequence,
    pub total_transfers: u8,
    pub get_off_stop_id: IdType,
    pub boarding_stop_id: IdType,
    pub previous_transfer: Option<Id<InProgressTrip>>,
    pub is_free_transfer: bool
}
