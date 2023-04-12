use crate::gtfs_wrapper::RouteType;
use crate::time::Time;
use crate::web::LatLng;
use std::collections::HashSet;

pub struct Configuration {
    pub start_time: Time,
    pub duration_secs: f64,
    pub location: LatLng,
    pub agency_ids: HashSet<u8>,
    pub modes: Vec<RouteType>,
}
