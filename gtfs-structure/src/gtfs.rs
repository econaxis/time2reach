use crate::{objects::*, Error, RawGtfs};
use chrono::prelude::NaiveDate;
use chrono::Duration;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::Arc;

/// Data structure with all the GTFS objects
///
/// This structure is easier to use than the [RawGtfs] structure as some relationships are parsed to be easier to use.
///
/// If you want to configure the behaviour (e.g. skipping : [StopTime]), see [crate::GtfsReader] for more personalisation
///
/// This is probably the entry point you want to use:
/// ```
/// let gtfs = gtfs_structures::Gtfs::new("fixtures/zips/gtfs.zip")?;
/// assert_eq!(gtfs.stops.len(), 5);
/// # Ok::<(), gtfs_structures::error::Error>(())
/// ```
///
/// The [StopTime] are accessible from the [Trip]
#[derive(Default)]
pub struct Gtfs {
    /// Time needed to read and parse the archive in milliseconds
    pub read_duration: i64,
    /// All Calendar by `service_id`
    pub calendar: HashMap<String, Calendar>,
    /// All calendar dates grouped by service_id
    pub calendar_dates: HashMap<String, Vec<CalendarDate>>,
    /// All stop by `stop_id`. Stops are in an [Arc] because they are also referenced by each [StopTime]
    pub stops: HashMap<String, Arc<Stop>>,
    /// All routes by `route_id`
    pub routes: HashMap<String, Route>,
    /// All trips by `trip_id`
    pub trips: HashMap<String, Trip>,
    /// All agencies. They can not be read by `agency_id`, as it is not a required field
    pub agencies: Vec<Agency>,
    /// All shapes by shape_id
    pub shapes: HashMap<String, Vec<Shape>>,
    /// All fare attributes by `fare_id`
    pub fare_attributes: HashMap<String, FareAttribute>,
    /// All feed information. There is no identifier
    pub feed_info: Vec<FeedInfo>,
}

impl TryFrom<RawGtfs> for Gtfs {
    type Error = Error;
    /// Tries to build a [Gtfs] from a [RawGtfs]
    ///
    /// It might fail if some mandatory files couldn’t be read or if there are references to other objects that are invalid.
    fn try_from(raw: RawGtfs) -> Result<Gtfs, Error> {
        let stops = to_stop_map(
            raw.stops?,
            raw.transfers.unwrap_or_else(|| Ok(Vec::new()))?,
            raw.pathways.unwrap_or(Ok(Vec::new()))?,
        )?;
        let frequencies = raw.frequencies.unwrap_or_else(|| Ok(Vec::new()))?;
        let trips = create_trips(raw.trips?, raw.stop_times?, frequencies, &stops)?;

        Ok(Gtfs {
            stops,
            routes: to_map(raw.routes?),
            trips,
            agencies: raw.agencies?,
            shapes: to_shape_map(raw.shapes.unwrap_or_else(|| Ok(Vec::new()))?),
            fare_attributes: to_map(raw.fare_attributes.unwrap_or_else(|| Ok(Vec::new()))?),
            feed_info: raw.feed_info.unwrap_or_else(|| Ok(Vec::new()))?,
            calendar: to_map(raw.calendar.unwrap_or_else(|| Ok(Vec::new()))?),
            calendar_dates: to_calendar_dates(
                raw.calendar_dates.unwrap_or_else(|| Ok(Vec::new()))?,
            ),
            read_duration: raw.read_duration,
        })
    }
}

impl Gtfs {
    /// Prints on stdout some basic statistics about the GTFS file (numbers of elements for each object). Mostly to be sure that everything was read
    pub fn print_stats(&self) {
        println!("GTFS data:");
        println!("  Read in {} ms", self.read_duration);
        println!("  Stops: {}", self.stops.len());
        println!("  Routes: {}", self.routes.len());
        println!("  Trips: {}", self.trips.len());
        println!("  Agencies: {}", self.agencies.len());
        println!("  Shapes: {}", self.shapes.len());
        println!("  Fare attributes: {}", self.fare_attributes.len());
        println!("  Feed info: {}", self.feed_info.len());
    }

    /// Reads from an url (if starts with `"http"`), or a local path (either a directory or zipped file)
    ///
    /// To read from an url, build with read-url feature
    /// See also [Gtfs::from_url] and [Gtfs::from_path] if you don’t want the library to guess
    pub fn new(gtfs: &str) -> Result<Gtfs, Error> {
        RawGtfs::new(gtfs).and_then(Gtfs::try_from)
    }

    /// Reads the GTFS from a local zip archive or local directory
    pub fn from_path<P>(path: P) -> Result<Gtfs, Error>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        RawGtfs::from_path(path).and_then(Gtfs::try_from)
    }

    /// Reads the GTFS from a remote url
    ///
    /// The library must be built with the read-url feature
    #[cfg(feature = "read-url")]
    pub fn from_url<U: reqwest::IntoUrl>(url: U) -> Result<Gtfs, Error> {
        RawGtfs::from_url(url).and_then(Gtfs::try_from)
    }

    /// Asynchronously reads the GTFS from a remote url
    ///
    /// The library must be built with the read-url feature
    #[cfg(feature = "read-url")]
    pub async fn from_url_async<U: reqwest::IntoUrl>(url: U) -> Result<Gtfs, Error> {
        RawGtfs::from_url_async(url).await.and_then(Gtfs::try_from)
    }

    /// Reads for any object implementing [std::io::Read] and [std::io::Seek]
    ///
    /// Mostly an internal function that abstracts reading from an url or local file
    pub fn from_reader<T: std::io::Read + std::io::Seek>(reader: T) -> Result<Gtfs, Error> {
        RawGtfs::from_reader(reader).and_then(Gtfs::try_from)
    }

    /// For a given a `service_id` and a starting date returns all the following day offset the vehicle runs
    ///
    /// For instance if the `start_date` is 2021-12-20, `[0, 4]` means that the vehicle will run the 20th and 24th
    ///
    /// It will consider use both [Calendar] and [CalendarDate] (both added and removed)
    pub fn trip_days(&self, service_id: &str, start_date: NaiveDate) -> Vec<u16> {
        let mut result = Vec::new();

        // Handle services given by specific days and exceptions
        let mut removed_days = HashSet::new();
        for extra_day in self
            .calendar_dates
            .get(service_id)
            .iter()
            .flat_map(|e| e.iter())
        {
            let offset = extra_day.date.signed_duration_since(start_date).num_days();
            if offset >= 0 {
                if extra_day.exception_type == Exception::Added {
                    result.push(offset as u16);
                } else if extra_day.exception_type == Exception::Deleted {
                    removed_days.insert(offset);
                }
            }
        }

        if let Some(calendar) = self.calendar.get(service_id) {
            let total_days = calendar
                .end_date
                .signed_duration_since(start_date)
                .num_days();
            for days_offset in 0..=total_days {
                let current_date = start_date + Duration::days(days_offset);

                if calendar.start_date <= current_date
                    && calendar.end_date >= current_date
                    && calendar.valid_weekday(current_date)
                    && !removed_days.contains(&days_offset)
                {
                    result.push(days_offset as u16);
                }
            }
        }

        result
    }

    /// Gets a [Stop] by its `stop_id`
    pub fn get_stop<'a>(&'a self, id: &str) -> Result<&'a Stop, Error> {
        match self.stops.get(id) {
            Some(stop) => Ok(stop),
            None => Err(Error::ReferenceError(id.to_owned())),
        }
    }

    /// Gets a [Trip] by its `trip_id`
    pub fn get_trip<'a>(&'a self, id: &str) -> Result<&'a Trip, Error> {
        self.trips
            .get(id)
            .ok_or_else(|| Error::ReferenceError(id.to_owned()))
    }

    /// Gets a [Route] by its `route_id`
    pub fn get_route<'a>(&'a self, id: &str) -> Result<&'a Route, Error> {
        self.routes
            .get(id)
            .ok_or_else(|| Error::ReferenceError(id.to_owned()))
    }

    /// Gets a [Calendar] by its `service_id`
    pub fn get_calendar<'a>(&'a self, id: &str) -> Result<&'a Calendar, Error> {
        self.calendar
            .get(id)
            .ok_or_else(|| Error::ReferenceError(id.to_owned()))
    }

    /// Gets all [CalendarDate] of a `service_id`
    pub fn get_calendar_date<'a>(&'a self, id: &str) -> Result<&'a Vec<CalendarDate>, Error> {
        self.calendar_dates
            .get(id)
            .ok_or_else(|| Error::ReferenceError(id.to_owned()))
    }

    /// Gets all [Shape] points of a `shape_id`
    pub fn get_shape<'a>(&'a self, id: &str) -> Result<&'a Vec<Shape>, Error> {
        self.shapes
            .get(id)
            .ok_or_else(|| Error::ReferenceError(id.to_owned()))
    }

    /// Gets a [FareAttribute] by its `fare_id`
    pub fn get_fare_attributes<'a>(&'a self, id: &str) -> Result<&'a FareAttribute, Error> {
        self.fare_attributes
            .get(id)
            .ok_or_else(|| Error::ReferenceError(id.to_owned()))
    }
}

fn to_map<O: Id>(elements: impl IntoIterator<Item = O>) -> HashMap<String, O> {
    elements
        .into_iter()
        .map(|e| (e.id().to_owned(), e))
        .collect()
}

fn to_stop_map(
    stops: Vec<Stop>,
    raw_transfers: Vec<RawTransfer>,
    raw_pathways: Vec<RawPathway>,
) -> Result<HashMap<String, Arc<Stop>>, Error> {
    let mut stop_map: HashMap<String, Stop> =
        stops.into_iter().map(|s| (s.id.clone(), s)).collect();

    for transfer in raw_transfers {
        stop_map
            .get(&transfer.to_stop_id)
            .ok_or_else(|| Error::ReferenceError(transfer.to_stop_id.to_string()))?;
        stop_map
            .entry(transfer.from_stop_id.clone())
            .and_modify(|stop| stop.transfers.push(StopTransfer::from(transfer)));
    }

    for pathway in raw_pathways {
        stop_map
            .get(&pathway.to_stop_id)
            .ok_or_else(|| Error::ReferenceError(pathway.to_stop_id.to_string()))?;
        stop_map
            .entry(pathway.from_stop_id.clone())
            .and_modify(|stop| stop.pathways.push(Pathway::from(pathway)));
    }

    let res = stop_map
        .into_iter()
        .map(|(i, s)| (i, Arc::new(s)))
        .collect();
    Ok(res)
}

fn to_shape_map(shapes: Vec<Shape>) -> HashMap<String, Vec<Shape>> {
    let mut res = HashMap::default();
    for s in shapes {
        let shape = res.entry(s.id.to_owned()).or_insert_with(Vec::new);
        shape.push(s);
    }
    // we sort the shape by it's pt_sequence
    for shapes in res.values_mut() {
        shapes.sort_by_key(|s| s.sequence);
    }

    res
}

fn to_calendar_dates(cd: Vec<CalendarDate>) -> HashMap<String, Vec<CalendarDate>> {
    let mut res = HashMap::default();
    for c in cd {
        let cal = res.entry(c.service_id.to_owned()).or_insert_with(Vec::new);
        cal.push(c);
    }
    res
}

fn create_trips(
    raw_trips: Vec<RawTrip>,
    raw_stop_times: Vec<RawStopTime>,
    raw_frequencies: Vec<RawFrequency>,
    stops: &HashMap<String, Arc<Stop>>,
) -> Result<HashMap<String, Trip>, Error> {
    let mut trips = to_map(raw_trips.into_iter().map(|rt| Trip {
        id: rt.id,
        service_id: rt.service_id,
        route_id: rt.route_id,
        stop_times: vec![],
        shape_id: rt.shape_id,
        trip_headsign: rt.trip_headsign,
        trip_short_name: rt.trip_short_name,
        direction_id: rt.direction_id,
        block_id: rt.block_id,
        wheelchair_accessible: rt.wheelchair_accessible,
        bikes_allowed: rt.bikes_allowed,
        frequencies: vec![],
    }));
    for s in raw_stop_times {
        let trip = &mut trips
            .get_mut(&s.trip_id)
            .ok_or_else(|| Error::ReferenceError(s.trip_id.to_string()))?;
        let stop = stops
            .get(&s.stop_id)
            .ok_or_else(|| Error::ReferenceError(s.stop_id.to_string()))?;
        trip.stop_times.push(StopTime::from(&s, Arc::clone(stop)));
    }

    for trip in &mut trips.values_mut() {
        trip.stop_times
            .sort_by(|a, b| a.stop_sequence.cmp(&b.stop_sequence));
    }

    for f in raw_frequencies {
        let trip = &mut trips
            .get_mut(&f.trip_id)
            .ok_or_else(|| Error::ReferenceError(f.trip_id.to_string()))?;
        trip.frequencies.push(Frequency::from(&f));
    }

    Ok(trips)
}
