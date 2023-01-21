use crate::objects::*;
use crate::Error;
use crate::GtfsReader;
use std::path::Path;

/// Data structure that map the GTFS csv with little intelligence
///
/// This is used to analyze the GTFS and detect anomalies
/// To manipulate the transit data, maybe [crate::Gtfs] will be more convienient
#[derive(Debug)]
pub struct RawGtfs {
    /// Time needed to read and parse the archive in milliseconds
    pub read_duration: i64,
    /// All Calendar, None if the file was absent as it is not mandatory
    pub calendar: Option<Result<Vec<Calendar>, Error>>,
    /// All Calendar dates, None if the file was absent as it is not mandatory
    pub calendar_dates: Option<Result<Vec<CalendarDate>, Error>>,
    /// All Stops
    pub stops: Result<Vec<Stop>, Error>,
    /// All Routes
    pub routes: Result<Vec<Route>, Error>,
    /// All Trips
    pub trips: Result<Vec<RawTrip>, Error>,
    /// All Agencies
    pub agencies: Result<Vec<Agency>, Error>,
    /// All shapes points, None if the file was absent as it is not mandatory
    pub shapes: Option<Result<Vec<Shape>, Error>>,
    /// All FareAttribates, None if the file was absent as it is not mandatory
    pub fare_attributes: Option<Result<Vec<FareAttribute>, Error>>,
    /// All Frequencies, None if the file was absent as it is not mandatory
    pub frequencies: Option<Result<Vec<RawFrequency>, Error>>,
    /// All Transfers, None if the file was absent as it is not mandatory
    pub transfers: Option<Result<Vec<RawTransfer>, Error>>,
    /// All Pathways, None if the file was absent as it is not mandatory
    pub pathways: Option<Result<Vec<RawPathway>, Error>>,
    /// All FeedInfo, None if the file was absent as it is not mandatory
    pub feed_info: Option<Result<Vec<FeedInfo>, Error>>,
    /// All StopTimes
    pub stop_times: Result<Vec<RawStopTime>, Error>,
    /// All files that are present in the feed
    pub files: Vec<String>,
    /// sha256 sum of the feed
    pub sha256: Option<String>,
}

impl RawGtfs {
    /// Prints on stdout some basic statistics about the GTFS file (numbers of elements for each object). Mostly to be sure that everything was read
    pub fn print_stats(&self) {
        println!("GTFS data:");
        println!("  Read in {} ms", self.read_duration);
        println!("  Stops: {}", mandatory_file_summary(&self.stops));
        println!("  Routes: {}", mandatory_file_summary(&self.routes));
        println!("  Trips: {}", mandatory_file_summary(&self.trips));
        println!("  Agencies: {}", mandatory_file_summary(&self.agencies));
        println!("  Stop times: {}", mandatory_file_summary(&self.stop_times));
        println!("  Shapes: {}", optional_file_summary(&self.shapes));
        println!("  Fares: {}", optional_file_summary(&self.fare_attributes));
        println!(
            "  Frequencies: {}",
            optional_file_summary(&self.frequencies)
        );
        println!("  Transfers: {}", optional_file_summary(&self.transfers));
        println!("  Pathways: {}", optional_file_summary(&self.pathways));
        println!("  Feed info: {}", optional_file_summary(&self.feed_info));
    }

    /// Reads from an url (if starts with http), or a local path (either a directory or zipped file)
    ///
    /// To read from an url, build with read-url feature
    /// See also [RawGtfs::from_url] and [RawGtfs::from_path] if you don’t want the library to guess
    pub fn new(gtfs: &str) -> Result<Self, Error> {
        GtfsReader::default().raw().read(gtfs)
    }

    /// Reads the raw GTFS from a local zip archive or local directory
    pub fn from_path<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path> + std::fmt::Display,
    {
        GtfsReader::default().raw().read_from_path(path)
    }

    /// Reads the raw GTFS from a remote url
    ///
    /// The library must be built with the read-url feature
    #[cfg(feature = "read-url")]
    pub fn from_url<U: reqwest::IntoUrl>(url: U) -> Result<Self, Error> {
        GtfsReader::default().raw().read_from_url(url)
    }

    /// Non-blocking read the raw GTFS from a remote url
    ///
    /// The library must be built with the read-url feature
    #[cfg(feature = "read-url")]
    pub async fn from_url_async<U: reqwest::IntoUrl>(url: U) -> Result<Self, Error> {
        GtfsReader::default().raw().read_from_url_async(url).await
    }

    /// Reads for any object implementing [std::io::Read] and [std::io::Seek]
    ///
    /// Mostly an internal function that abstracts reading from an url or local file
    pub fn from_reader<T: std::io::Read + std::io::Seek>(reader: T) -> Result<Self, Error> {
        GtfsReader::default().raw().read_from_reader(reader)
    }

    pub(crate) fn unknown_to_default(&mut self) {
        if let Ok(stops) = &mut self.stops {
            for stop in stops.iter_mut() {
                if let LocationType::Unknown(_) = stop.location_type {
                    stop.location_type = LocationType::default();
                }
                if let Availability::Unknown(_) = stop.wheelchair_boarding {
                    stop.wheelchair_boarding = Availability::default();
                }
            }
        }
        if let Ok(stop_times) = &mut self.stop_times {
            for stop_time in stop_times.iter_mut() {
                if let PickupDropOffType::Unknown(_) = stop_time.pickup_type {
                    stop_time.pickup_type = PickupDropOffType::default();
                }
                if let PickupDropOffType::Unknown(_) = stop_time.drop_off_type {
                    stop_time.drop_off_type = PickupDropOffType::default();
                }
                if let ContinuousPickupDropOff::Unknown(_) = stop_time.continuous_pickup {
                    stop_time.continuous_pickup = ContinuousPickupDropOff::default();
                }
                if let ContinuousPickupDropOff::Unknown(_) = stop_time.continuous_drop_off {
                    stop_time.continuous_drop_off = ContinuousPickupDropOff::default();
                }
            }
        }
        if let Ok(trips) = &mut self.trips {
            for trip in trips.iter_mut() {
                if let Availability::Unknown(_) = trip.wheelchair_accessible {
                    trip.wheelchair_accessible = Availability::default();
                }
                if let BikesAllowedType::Unknown(_) = trip.bikes_allowed {
                    trip.bikes_allowed = BikesAllowedType::default();
                }
            }
        }
    }
}

fn mandatory_file_summary<T>(objs: &Result<Vec<T>, Error>) -> String {
    match objs {
        Ok(vec) => format!("{} objects", vec.len()),
        Err(e) => format!("Could not read {}", e),
    }
}

fn optional_file_summary<T>(objs: &Option<Result<Vec<T>, Error>>) -> String {
    match objs {
        Some(objs) => mandatory_file_summary(objs),
        None => "File not present".to_string(),
    }
}
