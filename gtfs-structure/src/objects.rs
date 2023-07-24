pub use crate::enums::*;
use crate::serde_helpers::*;
use chrono::{Datelike, NaiveDate, Weekday};
use rgb::RGB8;

use std::fmt;
use std::sync::Arc;

/// Objects that have an identifier implement this trait
///
/// Those identifier are technical and should not be shown to travellers
pub trait Id {
    /// Identifier of the object
    fn id(&self) -> &str;
}

impl<T: Id> Id for Arc<T> {
    fn id(&self) -> &str {
        self.as_ref().id()
    }
}

/// Trait to introspect what is the object’s type (stop, route…)
pub trait Type {
    /// What is the type of the object
    fn object_type(&self) -> ObjectType;
}

impl<T: Type> Type for Arc<T> {
    fn object_type(&self) -> ObjectType {
        self.as_ref().object_type()
    }
}

/// A calender describes on which days the vehicle runs. See <https://gtfs.org/reference/static/#calendartxt>
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Calendar {
    /// Unique technical identifier (not for the traveller) of this calendar
    #[serde(rename = "service_id")]
    pub id: String,
    /// Does the service run on mondays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub monday: bool,
    /// Does the service run on tuesdays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub tuesday: bool,
    /// Does the service run on wednesdays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub wednesday: bool,
    /// Does the service run on thursdays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub thursday: bool,
    /// Does the service run on fridays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub friday: bool,
    /// Does the service run on saturdays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub saturday: bool,
    /// Does the service run on sundays
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub sunday: bool,
    /// Start service day for the service interval
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub start_date: NaiveDate,
    /// End service day for the service interval. This service day is included in the interval
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub end_date: NaiveDate,
}

impl Type for Calendar {
    fn object_type(&self) -> ObjectType {
        ObjectType::Calendar
    }
}

impl Id for Calendar {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}—{}", self.start_date, self.end_date)
    }
}

impl Calendar {
    /// Returns true if there is a service running on that day
    pub fn valid_weekday(&self, date: NaiveDate) -> bool {
        match date.weekday() {
            Weekday::Mon => self.monday,
            Weekday::Tue => self.tuesday,
            Weekday::Wed => self.wednesday,
            Weekday::Thu => self.thursday,
            Weekday::Fri => self.friday,
            Weekday::Sat => self.saturday,
            Weekday::Sun => self.sunday,
        }
    }
}

/// Defines a specific date that can be added or removed from a [Calendar]. See <https://gtfs.org/reference/static/#calendar_datestxt>
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CalendarDate {
    /// Identifier of the service that is modified at this date
    pub service_id: String,
    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    /// Date where the service will be added or deleted
    pub date: NaiveDate,
    /// Is the service added or deleted
    pub exception_type: Exception,
}

/// A physical stop, station or area. See <https://gtfs.org/reference/static/#stopstxt>
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Stop {
    /// Unique technical identifier (not for the traveller) of the stop
    #[serde(rename = "stop_id")]
    pub id: String,
    /// Short text or a number that identifies the location for riders
    #[serde(rename = "stop_code")]
    pub code: Option<String>,
    ///Name of the location. Use a name that people will understand in the local and tourist vernacular
    #[serde(rename = "stop_name")]
    pub name: String,
    /// Description of the location that provides useful, quality information
    #[serde(default, rename = "stop_desc")]
    pub description: String,
    /// Type of the location
    #[serde(default)]
    pub location_type: LocationType,
    /// Defines hierarchy between the different locations
    pub parent_station: Option<String>,
    /// Identifies the fare zone for a stop
    pub zone_id: Option<String>,
    /// URL of a web page about the location
    #[serde(rename = "stop_url")]
    pub url: Option<String>,
    /// Longitude of the stop
    #[serde(deserialize_with = "de_with_optional_float")]
    #[serde(serialize_with = "serialize_float_as_str")]
    #[serde(rename = "stop_lon", default)]
    pub longitude: Option<f64>,
    /// Latitude of the stop
    #[serde(deserialize_with = "de_with_optional_float")]
    #[serde(serialize_with = "serialize_float_as_str")]
    #[serde(rename = "stop_lat", default)]
    pub latitude: Option<f64>,
    /// Timezone of the location
    #[serde(rename = "stop_timezone")]
    pub timezone: Option<String>,
    /// Indicates whether wheelchair boardings are possible from the location
    #[serde(deserialize_with = "de_with_empty_default", default)]
    pub wheelchair_boarding: Availability,
    /// Level of the location. The same level can be used by multiple unlinked stations
    pub level_id: Option<String>,
    /// Platform identifier for a platform stop (a stop belonging to a station)
    pub platform_code: Option<String>,
    /// Transfers from this Stop
    #[serde(skip)]
    pub transfers: Vec<StopTransfer>,
    /// Pathways from this stop
    #[serde(skip)]
    pub pathways: Vec<Pathway>,
}

impl Type for Stop {
    fn object_type(&self) -> ObjectType {
        ObjectType::Stop
    }
}

impl Id for Stop {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A [StopTime] where the relations with [Trip] and [Stop] have not been tested
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RawStopTime {
    /// [Trip] to which this stop time belongs to
    pub trip_id: String,
    /// Arrival time of the stop time.
    /// It's an option since the intermediate stops can have have no arrival
    /// and this arrival needs to be interpolated
    #[serde(
        deserialize_with = "deserialize_optional_time",
        serialize_with = "serialize_optional_time"
    )]
    pub arrival_time: Option<u32>,
    /// Departure time of the stop time.
    /// It's an option since the intermediate stops can have have no departure
    /// and this departure needs to be interpolated
    #[serde(
        deserialize_with = "deserialize_optional_time",
        serialize_with = "serialize_optional_time"
    )]
    pub departure_time: Option<u32>,
    /// Identifier of the [Stop] where the vehicle stops
    pub stop_id: String,
    /// Order of stops for a particular trip. The values must increase along the trip but do not need to be consecutive
    pub stop_sequence: u16,
    /// Text that appears on signage identifying the trip's destination to riders
    pub stop_headsign: Option<String>,
    /// Indicates pickup method
    #[serde(default)]
    pub pickup_type: PickupDropOffType,
    /// Indicates drop off method
    #[serde(default)]
    pub drop_off_type: PickupDropOffType,
    /// Indicates whether a rider can board the transit vehicle anywhere along the vehicle’s travel path
    #[serde(default)]
    pub continuous_pickup: ContinuousPickupDropOff,
    /// Indicates whether a rider can alight from the transit vehicle at any point along the vehicle’s travel path
    #[serde(default)]
    pub continuous_drop_off: ContinuousPickupDropOff,
    /// Actual distance traveled along the associated shape, from the first stop to the stop specified in this record. This field specifies how much of the shape to draw between any two stops during a trip
    pub shape_dist_traveled: Option<f32>,
    /// Indicates if arrival and departure times for a stop are strictly adhered to by the vehicle or if they are instead approximate and/or interpolated times
    #[serde(default)]
    pub timepoint: TimepointType,
}

/// The moment where a vehicle, running on [Trip] stops at a [Stop]. See <https://gtfs.org/reference/static/#stopstxt>
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StopTime {
    /// Arrival time of the stop time.
    /// It's an option since the intermediate stops can have have no arrival
    /// and this arrival needs to be interpolated
    pub arrival_time: Option<u32>,
    /// [Stop] where the vehicle stops
    pub stop: Arc<Stop>,
    /// Departure time of the stop time.
    /// It's an option since the intermediate stops can have have no departure
    /// and this departure needs to be interpolated
    pub departure_time: Option<u32>,
    /// Indicates pickup method
    pub pickup_type: PickupDropOffType,
    /// Indicates drop off method
    pub drop_off_type: PickupDropOffType,
    /// Order of stops for a particular trip. The values must increase along the trip but do not need to be consecutive
    pub stop_sequence: u16,
    /// Text that appears on signage identifying the trip's destination to riders
    pub stop_headsign: Option<String>,
    /// Indicates whether a rider can board the transit vehicle anywhere along the vehicle’s travel path
    pub continuous_pickup: ContinuousPickupDropOff,
    /// Indicates whether a rider can alight from the transit vehicle at any point along the vehicle’s travel path
    pub continuous_drop_off: ContinuousPickupDropOff,
    /// Actual distance traveled along the associated shape, from the first stop to the stop specified in this record. This field specifies how much of the shape to draw between any two stops during a trip
    pub shape_dist_traveled: Option<f32>,
    /// Indicates if arrival and departure times for a stop are strictly adhered to by the vehicle or if they are instead approximate and/or interpolated times
    pub timepoint: TimepointType,
}

impl StopTime {
    /// Creates [StopTime] by linking a [RawStopTime::stop_id] to the actual [Stop]
    pub fn from(stop_time_gtfs: &RawStopTime, stop: Arc<Stop>) -> Self {
        Self {
            arrival_time: stop_time_gtfs.arrival_time,
            departure_time: stop_time_gtfs.departure_time,
            stop,
            pickup_type: stop_time_gtfs.pickup_type,
            drop_off_type: stop_time_gtfs.drop_off_type,
            stop_sequence: stop_time_gtfs.stop_sequence,
            stop_headsign: stop_time_gtfs.stop_headsign.clone(),
            continuous_pickup: stop_time_gtfs.continuous_pickup,
            continuous_drop_off: stop_time_gtfs.continuous_drop_off,
            shape_dist_traveled: stop_time_gtfs.shape_dist_traveled,
            timepoint: stop_time_gtfs.timepoint,
        }
    }
}

/// A route is a commercial line (there can be various stop sequences for a same line). See <https://gtfs.org/reference/static/#routestxt>
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Route {
    /// Unique technical (not for the traveller) identifier for the route
    #[serde(rename = "route_id")]
    pub id: String,
    /// Short name of a route. This will often be a short, abstract identifier like "32", "100X", or "Green" that riders use to identify a route, but which doesn't give any indication of what places the route serves
    #[serde(rename = "route_short_name")]
    pub short_name: String,
    /// Full name of a route. This name is generally more descriptive than the [Route::short_name]] and often includes the route's destination or stop
    #[serde(rename = "route_long_name")]
    pub long_name: String,
    /// Description of a route that provides useful, quality information
    #[serde(rename = "route_desc")]
    pub desc: Option<String>,
    /// Indicates the type of transportation used on a route
    pub route_type: RouteType,
    /// URL of a web page about the particular route
    #[serde(rename = "route_url")]
    pub url: Option<String>,
    /// Agency for the specified route
    pub agency_id: Option<String>,
    /// Orders the routes in a way which is ideal for presentation to customers. Routes with smaller route_sort_order values should be displayed first.
    #[serde(rename = "route_sort_order")]
    pub order: Option<u32>,
    /// Route color designation that matches public facing material
    #[serde(
        deserialize_with = "deserialize_route_color",
        serialize_with = "serialize_color",
        rename = "route_color",
        default = "default_route_color"
    )]
    pub color: RGB8,
    /// Legible color to use for text drawn against a background of [Route::route_color]
    #[serde(
        deserialize_with = "deserialize_route_text_color",
        serialize_with = "serialize_color",
        rename = "route_text_color",
        default
    )]
    pub text_color: RGB8,
    /// Indicates whether a rider can board the transit vehicle anywhere along the vehicle’s travel path
    #[serde(default)]
    pub continuous_pickup: ContinuousPickupDropOff,
    /// Indicates whether a rider can alight from the transit vehicle at any point along the vehicle’s travel path
    #[serde(default)]
    pub continuous_drop_off: ContinuousPickupDropOff,
}

impl Type for Route {
    fn object_type(&self) -> ObjectType {
        ObjectType::Route
    }
}

impl Id for Route {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.long_name.is_empty() {
            write!(f, "{}", self.long_name)
        } else {
            write!(f, "{}", self.short_name)
        }
    }
}

/// A [Trip] where the relationships with other objects have not been checked
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RawTrip {
    /// Unique technical (not for the traveller) identifier for the Trip
    #[serde(rename = "trip_id")]
    pub id: String,
    /// References the [Calendar] on which this trip runs
    pub service_id: String,
    /// References along which [Route] this trip runs
    pub route_id: String,
    /// Shape of the trip
    pub shape_id: Option<String>,
    /// Text that appears on signage identifying the trip's destination to riders
    pub trip_headsign: Option<String>,
    /// Public facing text used to identify the trip to riders, for instance, to identify train numbers for commuter rail trips
    pub trip_short_name: Option<String>,
    /// Indicates the direction of travel for a trip. This field is not used in routing; it provides a way to separate trips by direction when publishing time tables
    pub direction_id: Option<DirectionType>,
    /// Identifies the block to which the trip belongs. A block consists of a single trip or many sequential trips made using the same vehicle, defined by shared service days and block_id. A block_id can have trips with different service days, making distinct blocks
    pub block_id: Option<String>,
    /// Indicates wheelchair accessibility
    #[serde(default)]
    pub wheelchair_accessible: Availability,
    /// Indicates whether bikes are allowed
    #[serde(default)]
    pub bikes_allowed: BikesAllowedType,
}

impl Type for RawTrip {
    fn object_type(&self) -> ObjectType {
        ObjectType::Trip
    }
}

impl Id for RawTrip {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for RawTrip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "route id: {}, service id: {}",
            self.route_id, self.service_id
        )
    }
}

/// A Trip is a vehicle that follows a sequence of [StopTime] on certain days. See <https://gtfs.org/reference/static/#tripstxt>
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Trip {
    /// Unique technical identifier (not for the traveller) for the Trip
    pub id: String,
    /// References the [Calendar] on which this trip runs
    pub service_id: String,
    /// References along which [Route] this trip runs
    pub route_id: String,
    /// All the [StopTime] that define the trip
    pub stop_times: Vec<StopTime>,
    /// Text that appears on signage identifying the trip's destination to riders
    pub shape_id: Option<String>,
    /// Text that appears on signage identifying the trip's destination to riders
    pub trip_headsign: Option<String>,
    /// Public facing text used to identify the trip to riders, for instance, to identify train numbers for commuter rail trips
    pub trip_short_name: Option<String>,
    /// Indicates the direction of travel for a trip. This field is not used in routing; it provides a way to separate trips by direction when publishing time tables
    pub direction_id: Option<DirectionType>,
    /// Identifies the block to which the trip belongs. A block consists of a single trip or many sequential trips made using the same vehicle, defined by shared service days and block_id. A block_id can have trips with different service days, making distinct blocks
    pub block_id: Option<String>,
    /// Indicates wheelchair accessibility
    pub wheelchair_accessible: Availability,
    /// Indicates whether bikes are allowed
    pub bikes_allowed: BikesAllowedType,
    /// During which periods the trip runs by frequency and not by fixed timetable
    pub frequencies: Vec<Frequency>,
}

impl Type for Trip {
    fn object_type(&self) -> ObjectType {
        ObjectType::Trip
    }
}

impl Id for Trip {
    fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for Trip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "route id: {}, service id: {}",
            self.route_id, self.service_id
        )
    }
}

/// General informations about the agency running the network. See <https://gtfs.org/reference/static/#agencytxt>
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Agency {
    /// Unique technical (not for the traveller) identifier for the Agency
    #[serde(rename = "agency_id")]
    pub id: Option<String>,
    ///Full name of the transit agency
    #[serde(rename = "agency_name")]
    pub name: String,
    /// Full name of the transit agency.
    #[serde(rename = "agency_url")]
    pub url: String,
    /// Timezone where the transit agency is located
    #[serde(rename = "agency_timezone")]
    pub timezone: String,
    /// Primary language used by this transit agency
    #[serde(rename = "agency_lang")]
    pub lang: Option<String>,
    /// A voice telephone number for the specified agency
    #[serde(rename = "agency_phone")]
    pub phone: Option<String>,
    /// URL of a web page that allows a rider to purchase tickets or other fare instruments for that agency online
    #[serde(rename = "agency_fare_url")]
    pub fare_url: Option<String>,
    /// Email address actively monitored by the agency’s customer service department
    #[serde(rename = "agency_email")]
    pub email: Option<String>,
}

impl Type for Agency {
    fn object_type(&self) -> ObjectType {
        ObjectType::Agency
    }
}

impl Id for Agency {
    fn id(&self) -> &str {
        match &self.id {
            None => "",
            Some(id) => id,
        }
    }
}

impl fmt::Display for Agency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A single geographical point decribing the shape of a [Trip]. See <https://gtfs.org/reference/static/#shapestxt>
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Shape {
    /// Unique technical (not for the traveller) identifier for the Shape
    #[serde(rename = "shape_id")]
    pub id: String,
    #[serde(rename = "shape_pt_lat", default)]
    /// Latitude of a shape point
    pub latitude: f64,
    /// Longitude of a shape point
    #[serde(rename = "shape_pt_lon", default)]
    pub longitude: f64,
    /// Sequence in which the shape points connect to form the shape. Values increase along the trip but do not need to be consecutive.
    #[serde(rename = "shape_pt_sequence")]
    pub sequence: usize,
    /// Actual distance traveled along the shape from the first shape point to the point specified in this record. Used by trip planners to show the correct portion of the shape on a map
    #[serde(rename = "shape_dist_traveled")]
    pub dist_traveled: Option<f32>,
}

impl Type for Shape {
    fn object_type(&self) -> ObjectType {
        ObjectType::Shape
    }
}

impl Id for Shape {
    fn id(&self) -> &str {
        &self.id
    }
}

/// Defines one possible fare. See <https://gtfs.org/reference/static/#fare_attributestxt>
#[derive(Debug, Serialize, Deserialize)]
pub struct FareAttribute {
    /// Unique technical (not for the traveller) identifier for the FareAttribute
    #[serde(rename = "fare_id")]
    pub id: String,
    /// Fare price, in the unit specified by [FareAttribute::currency]
    pub price: String,
    /// Currency used to pay the fare.
    #[serde(rename = "currency_type")]
    pub currency: String,
    ///Indicates when the fare must be paid
    pub payment_method: PaymentMethod,
    /// Indicates the number of transfers permitted on this fare
    pub transfers: Transfers,
    /// Identifies the relevant agency for a fare
    pub agency_id: Option<String>,
    /// Length of time in seconds before a transfer expires
    pub transfer_duration: Option<usize>,
}

impl Id for FareAttribute {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Type for FareAttribute {
    fn object_type(&self) -> ObjectType {
        ObjectType::Fare
    }
}

/// A [Frequency] before being merged into the corresponding [Trip]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawFrequency {
    /// References the [Trip] that uses frequency
    pub trip_id: String,
    /// Time at which the first vehicle departs from the first stop of the trip
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub start_time: u32,
    /// Time at which service changes to a different headway (or ceases) at the first stop in the trip
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub end_time: u32,
    /// Time, in seconds, between departures from the same stop (headway) for the trip, during the time interval specified by start_time and end_time
    pub headway_secs: u32,
    /// Indicates the type of service for a trip
    pub exact_times: Option<ExactTimes>,
}

/// Timetables can be defined by the frequency of their vehicles. See <<https://gtfs.org/reference/static/#frequenciestxt>>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Frequency {
    /// Time at which the first vehicle departs from the first stop of the trip
    pub start_time: u32,
    /// Time at which service changes to a different headway (or ceases) at the first stop in the trip
    pub end_time: u32,
    /// Time, in seconds, between departures from the same stop (headway) for the trip, during the time interval specified by start_time and end_time
    pub headway_secs: u32,
    /// Indicates the type of service for a trip
    pub exact_times: Option<ExactTimes>,
}

impl Frequency {
    /// Converts from a [RawFrequency] to a [Frequency]
    pub fn from(frequency: &RawFrequency) -> Self {
        Self {
            start_time: frequency.start_time,
            end_time: frequency.end_time,
            headway_secs: frequency.headway_secs,
            exact_times: frequency.exact_times,
        }
    }
}

/// Transfer information between stops before merged into [Stop]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawTransfer {
    /// Stop from which to leave
    pub from_stop_id: String,
    /// Stop which to transfer to
    pub to_stop_id: String,
    /// Type of the transfer
    pub transfer_type: TransferType,
    /// Minimum time needed to make the transfer in seconds
    pub min_transfer_time: Option<u32>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
/// Transfer information between stops
pub struct StopTransfer {
    /// Stop which to transfer to
    pub to_stop_id: String,
    /// Type of the transfer
    pub transfer_type: TransferType,
    /// Minimum time needed to make the transfer in seconds
    pub min_transfer_time: Option<u32>,
}

impl From<RawTransfer> for StopTransfer {
    /// Converts from a [RawTransfer] to a [StopTransfer]
    fn from(transfer: RawTransfer) -> Self {
        Self {
            to_stop_id: transfer.to_stop_id,
            transfer_type: transfer.transfer_type,
            min_transfer_time: transfer.min_transfer_time,
        }
    }
}

/// Meta-data about the feed. See <https://gtfs.org/reference/static/#feed_infotxt>
#[derive(Debug, Serialize, Deserialize)]
pub struct FeedInfo {
    /// Full name of the organization that publishes the dataset.
    #[serde(rename = "feed_publisher_name")]
    pub name: String,
    /// URL of the dataset publishing organization's website
    #[serde(rename = "feed_publisher_url")]
    pub url: String,
    /// Default language used for the text in this dataset
    #[serde(rename = "feed_lang")]
    pub lang: String,
    /// Defines the language that should be used when the data consumer doesn’t know the language of the rider
    pub default_lang: Option<String>,
    /// The dataset provides complete and reliable schedule information for service in the period from this date
    #[serde(
        deserialize_with = "deserialize_option_date",
        serialize_with = "serialize_option_date",
        rename = "feed_start_date",
        default
    )]
    pub start_date: Option<NaiveDate>,
    ///The dataset provides complete and reliable schedule information for service in the period until this date
    #[serde(
        deserialize_with = "deserialize_option_date",
        serialize_with = "serialize_option_date",
        rename = "feed_end_date",
        default
    )]
    pub end_date: Option<NaiveDate>,
    /// String that indicates the current version of their GTFS dataset
    #[serde(rename = "feed_version")]
    pub version: Option<String>,
    /// Email address for communication regarding the GTFS dataset and data publishing practices
    #[serde(rename = "feed_contact_email")]
    pub contact_email: Option<String>,
    /// URL for contact information, a web-form, support desk, or other tools for communication regarding the GTFS dataset and data publishing practices
    #[serde(rename = "feed_contact_url")]
    pub contact_url: Option<String>,
}

impl fmt::Display for FeedInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A graph representation to describe subway or train, with nodes (the locations) and edges (the pathways).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawPathway {
    /// Uniquely identifies the pathway
    #[serde(rename = "pathway_id")]
    pub id: String,
    /// Location at which the pathway begins
    pub from_stop_id: String,
    /// Location at which the pathway ends
    pub to_stop_id: String,
    /// Type of pathway between the specified (from_stop_id, to_stop_id) pair
    #[serde(rename = "pathway_mode")]
    pub mode: PathwayMode,
    /// Indicates in which direction the pathway can be used
    pub is_bidirectional: PathwayDirectionType,
    /// Horizontal length in meters of the pathway from the origin location to the destination location
    pub length: Option<f32>,
    /// Average time in seconds needed to walk through the pathway from the origin location to the destination location
    pub traversal_time: Option<u32>,
    /// Number of stairs of the pathway
    pub stair_count: Option<u32>,
    /// Maximum slope ratio of the pathway
    pub max_slope: Option<f32>,
    /// Minimum width of the pathway in meters
    pub min_width: Option<f32>,
    /// String of text from physical signage visible to transit riders
    pub signposted_as: Option<String>,
    /// Same than the signposted_as field, but when the pathways is used backward
    pub reversed_signposted_as: Option<String>,
}

/// Pathway going from a stop to another.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pathway {
    /// Uniquely identifies the pathway
    pub id: String,
    /// Location at which the pathway ends
    pub to_stop_id: String,
    /// Type of pathway between the specified (from_stop_id, to_stop_id) pair
    pub mode: PathwayMode,
    /// Indicates in which direction the pathway can be used
    pub is_bidirectional: PathwayDirectionType,
    /// Horizontal length in meters of the pathway from the origin location to the destination location
    pub length: Option<f32>,
    /// Average time in seconds needed to walk through the pathway from the origin location to the destination location
    pub traversal_time: Option<u32>,
    /// Number of stairs of the pathway
    pub stair_count: Option<u32>,
    /// Maximum slope ratio of the pathway
    pub max_slope: Option<f32>,
    /// Minimum width of the pathway in meters
    pub min_width: Option<f32>,
    /// String of text from physical signage visible to transit riders
    pub signposted_as: Option<String>,
    /// Same than the signposted_as field, but when the pathways is used backward
    pub reversed_signposted_as: Option<String>,
}

impl Id for Pathway {
    fn id(&self) -> &str {
        &self.id
    }
}

impl From<RawPathway> for Pathway {
    /// Converts from a [RawPathway] to a [Pathway]
    fn from(raw: RawPathway) -> Self {
        Self {
            id: raw.id,
            to_stop_id: raw.to_stop_id,
            mode: raw.mode,
            is_bidirectional: raw.is_bidirectional,
            length: raw.length,
            max_slope: raw.max_slope,
            min_width: raw.min_width,
            reversed_signposted_as: raw.reversed_signposted_as,
            signposted_as: raw.signposted_as,
            stair_count: raw.stair_count,
            traversal_time: raw.traversal_time,
        }
    }
}
