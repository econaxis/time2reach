use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

/// All the objects type from the GTFS specification that this library reads
#[derive(Debug, Serialize, Eq, PartialEq, Hash)]
pub enum ObjectType {
    /// [Agency] <https://gtfs.org/reference/static/#agencytxt>
    Agency,
    /// [Stop] <https://gtfs.org/reference/static/#stopstxt>
    Stop,
    /// [Route] <https://gtfs.org/reference/static/#routestxt>
    Route,
    /// [Trip] <https://gtfs.org/reference/static/#tripstxt>
    Trip,
    /// [Calendar] <https://gtfs.org/reference/static/#calendartxt>
    Calendar,
    /// [Shape] <https://gtfs.org/reference/static/#shapestxt>
    Shape,
    /// [FareAttribute] <https://gtfs.org/reference/static/#fare_rulestxt>
    Fare,
}

/// Describes the kind of [Stop]. See <https://gtfs.org/reference/static/#stopstxt> `location_type`
#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default(bound = ""))]
pub enum LocationType {
    /// Stop (or Platform). A location where passengers board or disembark from a transit vehicle. Is called a platform when defined within a parent_station
    #[derivative(Default)]
    StopPoint,
    /// Station. A physical structure or area that contains one or more platform
    StopArea,
    /// A location where passengers can enter or exit a station from the street. If an entrance/exit belongs to multiple stations, it can be linked by pathways to both, but the data provider must pick one of them as parent
    StationEntrance,
    /// A location within a station, not matching any other [Stop::location_type], which can be used to link together pathways define in pathways.txt.
    GenericNode,
    /// A specific location on a platform, where passengers can board and/or alight vehicles
    BoardingArea,
    /// An unknown value
    Unknown(i32),
}

fn serialize_i32_as_str<S: Serializer>(s: S, value: i32) -> Result<S::Ok, S::Error> {
    s.serialize_str(&value.to_string())
}
impl<'de> Deserialize<'de> for LocationType {
    fn deserialize<D>(deserializer: D) -> Result<LocationType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "" | "0" => LocationType::StopPoint,
            "1" => LocationType::StopArea,
            "2" => LocationType::StationEntrance,
            "3" => LocationType::GenericNode,
            "4" => LocationType::BoardingArea,
            s => LocationType::Unknown(s.parse().map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid value for LocationType, must be an integer: {}",
                    s
                ))
            })?),
        })
    }
}

impl Serialize for LocationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serialize_i32_as_str(
            serializer,
            match self {
                LocationType::StopPoint => 0,
                LocationType::StopArea => 1,
                LocationType::StationEntrance => 2,
                LocationType::GenericNode => 3,
                LocationType::BoardingArea => 4,
                LocationType::Unknown(i) => *i,
            },
        )
    }
}

/// Describes the kind of [Route]. See <https://gtfs.org/reference/static/#routestxt> `route_type`
///
/// -ome route types are extended GTFS (<https://developers.google.com/transit/gtfs/reference/extended-route-types)>
#[derive(Debug, Derivative, Copy, Clone, PartialEq, Eq, Hash)]
#[derivative(Default(bound = ""))]
pub enum RouteType {
    /// Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area
    Tramway,
    /// Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area
    Subway,
    /// Used for intercity or long-distance travel
    Rail,
    /// Used for short- and long-distance bus routes
    #[derivative(Default)]
    Bus,
    /// Used for short- and long-distance boat service
    Ferry,
    /// Used for street-level rail cars where the cable runs beneath the vehicle, e.g., cable car in San Francisco
    CableCar,
    /// Aerial lift, suspended cable car (e.g., gondola lift, aerial tramway). Cable transport where cabins, cars, gondolas or open chairs are suspended by means of one or more cables
    Gondola,
    /// Any rail system designed for steep inclines
    Funicular,
    /// (extended) Used for intercity bus services
    Coach,
    /// (extended) Airplanes
    Air,
    /// (extended) Taxi, Cab
    Taxi,
    /// (extended) any other value
    Other(i32),
}

impl<'de> Deserialize<'de> for RouteType {
    fn deserialize<D>(deserializer: D) -> Result<RouteType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i = i32::deserialize(deserializer)?;

        let hundreds = i / 100;
        Ok(match (i, hundreds) {
            (0, _) | (_, 9) => RouteType::Tramway,
            (1, _) | (_, 4) => RouteType::Subway,
            (2, _) | (_, 1) => RouteType::Rail,
            (3, _) | (_, 7) | (_, 8) => RouteType::Bus,
            (4, _) | (_, 10) | (_, 12) => RouteType::Ferry,
            (5, _) => RouteType::CableCar,
            (6, _) | (_, 13) => RouteType::Gondola,
            (7, _) | (_, 14) => RouteType::Funicular,
            (_, 2) => RouteType::Coach,
            (_, 11) => RouteType::Air,
            (_, 15) => RouteType::Taxi,
            _ => RouteType::Other(i),
        })
    }
}

impl Serialize for RouteType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serializer.serialize_i32(match self {
            RouteType::Tramway => 0,
            RouteType::Subway => 1,
            RouteType::Rail => 2,
            RouteType::Bus => 3,
            RouteType::Ferry => 4,
            RouteType::CableCar => 5,
            RouteType::Gondola => 6,
            RouteType::Funicular => 7,
            RouteType::Coach => 200,
            RouteType::Air => 1100,
            RouteType::Taxi => 1500,
            RouteType::Other(i) => *i,
        })
    }
}

/// Describes if and how a traveller can board or alight the vehicle. See <https://gtfs.org/reference/static/#stop_timestxt> `pickup_type` and `dropoff_type`
#[derive(Debug, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default(bound = ""))]
pub enum PickupDropOffType {
    /// Regularly scheduled pickup or drop off (default when empty).
    #[derivative(Default)]
    Regular,
    /// No pickup or drop off available.
    NotAvailable,
    /// Must phone agency to arrange pickup or drop off.
    ArrangeByPhone,
    /// Must coordinate with driver to arrange pickup or drop off.
    CoordinateWithDriver,
    /// An unknown value not in the specification
    Unknown(i32),
}

impl<'de> Deserialize<'de> for PickupDropOffType {
    fn deserialize<D>(deserializer: D) -> Result<PickupDropOffType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "" | "0" => PickupDropOffType::Regular,
            "1" => PickupDropOffType::NotAvailable,
            "2" => PickupDropOffType::ArrangeByPhone,
            "3" => PickupDropOffType::CoordinateWithDriver,
            s => PickupDropOffType::Unknown(s.parse().map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid value for PickupDropOffType, must be an integer: {}",
                    s
                ))
            })?),
        })
    }
}

impl Serialize for PickupDropOffType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serialize_i32_as_str(
            serializer,
            match self {
                PickupDropOffType::Regular => 0,
                PickupDropOffType::NotAvailable => 1,
                PickupDropOffType::ArrangeByPhone => 2,
                PickupDropOffType::CoordinateWithDriver => 3,
                PickupDropOffType::Unknown(i) => *i,
            },
        )
    }
}

/// Indicates whether a rider can board the transit vehicle anywhere along the vehicle’s travel path
///
/// Those values are only defined on <https://developers.google.com/transit/gtfs/reference#routestxt,> not on <https://gtfs.org/reference/static/#routestxt>
#[derive(Debug, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default(bound = ""))]
pub enum ContinuousPickupDropOff {
    /// Continuous stopping pickup or drop off.
    Continuous,
    /// No continuous stopping pickup or drop off (default when empty).
    #[derivative(Default)]
    NotAvailable,
    /// Must phone agency to arrange continuous stopping pickup or drop off.
    ArrangeByPhone,
    /// Must coordinate with driver to arrange continuous stopping pickup or drop off.
    CoordinateWithDriver,
    /// An unknown value not in the specification
    Unknown(i32),
}

impl Serialize for ContinuousPickupDropOff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serialize_i32_as_str(
            serializer,
            match self {
                ContinuousPickupDropOff::Continuous => 0,
                ContinuousPickupDropOff::NotAvailable => 1,
                ContinuousPickupDropOff::ArrangeByPhone => 2,
                ContinuousPickupDropOff::CoordinateWithDriver => 3,
                ContinuousPickupDropOff::Unknown(i) => *i,
            },
        )
    }
}

impl<'de> Deserialize<'de> for ContinuousPickupDropOff {
    fn deserialize<D>(deserializer: D) -> Result<ContinuousPickupDropOff, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "0" => ContinuousPickupDropOff::Continuous,
            "" | "1" => ContinuousPickupDropOff::NotAvailable,
            "2" => ContinuousPickupDropOff::ArrangeByPhone,
            "3" => ContinuousPickupDropOff::CoordinateWithDriver,
            s => ContinuousPickupDropOff::Unknown(s.parse().map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid value for ContinuousPickupDropOff, must be an integer: {}",
                    s
                ))
            })?),
        })
    }
}

/// Describes if the stop time is exact or not. See <https://gtfs.org/reference/static/#stop_timestxt> `timepoint`
#[derive(Debug, Derivative, Serialize, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub enum TimepointType {
    /// Times are considered approximate
    #[serde(rename = "0")]
    Approximate = 0,
    /// Times are considered exact
    #[derivative(Default)]
    #[serde(rename = "1")]
    Exact = 1,
}

impl<'de> Deserialize<'de> for TimepointType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        match s.as_str() {
            "" | "1" => Ok(Self::Exact),
            "0" => Ok(Self::Approximate),
            v => Err(serde::de::Error::custom(format!(
                "invalid value for timepoint: {}",
                v
            ))),
        }
    }
}

/// Generic enum to define if a service (like wheelchair boarding) is available
#[derive(Debug, Derivative, PartialEq, Eq, Hash, Clone, Copy)]
#[derivative(Default)]
pub enum Availability {
    /// No information if the service is available
    #[derivative(Default)]
    InformationNotAvailable,
    /// The service is available
    Available,
    /// The service is not available
    NotAvailable,
    /// An unknown value not in the specification
    Unknown(i32),
}

impl<'de> Deserialize<'de> for Availability {
    fn deserialize<D>(deserializer: D) -> Result<Availability, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "" | "0" => Availability::InformationNotAvailable,
            "1" => Availability::Available,
            "2" => Availability::NotAvailable,
            s => Availability::Unknown(s.parse().map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid value for Availability, must be an integer: {}",
                    s
                ))
            })?),
        })
    }
}

impl Serialize for Availability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serialize_i32_as_str(
            serializer,
            match self {
                Availability::InformationNotAvailable => 0,
                Availability::Available => 1,
                Availability::NotAvailable => 2,
                Availability::Unknown(i) => *i,
            },
        )
    }
}

/// Defines if a [CalendarDate] is added or deleted from a [Calendar]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Exception {
    /// There will be a service on that day
    #[serde(rename = "1")]
    Added,
    /// There won’t be a service on that day
    #[serde(rename = "2")]
    Deleted,
}

/// Defines the direction of a [Trip], only for display, not for routing. See <https://gtfs.org/reference/static/#tripstxt> `direction_id`
#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum DirectionType {
    /// Travel in one direction (e.g. outbound travel).
    #[serde(rename = "0")]
    Outbound,
    /// Travel in the opposite direction (e.g. inbound travel).
    #[serde(rename = "1")]
    Inbound,
}

/// Is the [Trip] accessible with a bike. See <https://gtfs.org/reference/static/#tripstxt> `bikes_allowed`
#[derive(Debug, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default())]
pub enum BikesAllowedType {
    /// No bike information for the trip
    #[derivative(Default)]
    NoBikeInfo,
    /// Vehicle being used on this particular trip can accommodate at least one bicycle
    AtLeastOneBike,
    /// No bicycles are allowed on this trip
    NoBikesAllowed,
    /// An unknown value not in the specification
    Unknown(i32),
}

impl<'de> Deserialize<'de> for BikesAllowedType {
    fn deserialize<D>(deserializer: D) -> Result<BikesAllowedType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "" | "0" => BikesAllowedType::NoBikeInfo,
            "1" => BikesAllowedType::AtLeastOneBike,
            "2" => BikesAllowedType::NoBikesAllowed,
            s => BikesAllowedType::Unknown(s.parse().map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid value for BikeAllowedType, must be an integer: {}",
                    s
                ))
            })?),
        })
    }
}

impl Serialize for BikesAllowedType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: for extended route type, we might loose the initial precise route type
        serialize_i32_as_str(
            serializer,
            match self {
                BikesAllowedType::NoBikeInfo => 0,
                BikesAllowedType::AtLeastOneBike => 1,
                BikesAllowedType::NoBikesAllowed => 2,
                BikesAllowedType::Unknown(i) => *i,
            },
        )
    }
}

/// Defines where a [FareAttribute] can be paid
#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum PaymentMethod {
    /// Fare is paid on board
    #[serde(rename = "0")]
    Aboard,
    /// Fare must be paid before boarding
    #[serde(rename = "1")]
    PreBoarding,
}

/// Defines if the [Frequency] is exact (the vehicle runs exactly every n minutes) or not
#[derive(Debug, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum ExactTimes {
    /// Frequency-based trips
    FrequencyBased = 0,
    /// Schedule-based trips with the exact same headway throughout the day.
    ScheduleBased = 1,
}

impl<'de> Deserialize<'de> for ExactTimes {
    fn deserialize<D>(deserializer: D) -> Result<ExactTimes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "" | "0" => ExactTimes::FrequencyBased,
            "1" => ExactTimes::ScheduleBased,
            &_ => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid value `{}`, expected 0 or 1",
                    s
                )))
            }
        })
    }
}

/// Defines how many transfers can be done with on [FareAttribute]
#[derive(Debug, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default(bound = ""))]
pub enum Transfers {
    /// Unlimited transfers are permitted
    #[derivative(Default)]
    Unlimited,
    /// No transfers permitted on this fare
    NoTransfer,
    /// Riders may transfer once
    UniqueTransfer,
    ///Riders may transfer twice
    TwoTransfers,
    /// Other transfer values
    Other(i32),
}

impl<'de> Deserialize<'de> for Transfers {
    fn deserialize<D>(deserializer: D) -> Result<Transfers, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i = Option::<i32>::deserialize(deserializer)?;
        Ok(match i {
            Some(0) => Transfers::NoTransfer,
            Some(1) => Transfers::UniqueTransfer,
            Some(2) => Transfers::TwoTransfers,
            Some(a) => Transfers::Other(a),
            None => Transfers::default(),
        })
    }
}

impl Serialize for Transfers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Transfers::NoTransfer => serialize_i32_as_str(serializer, 0),
            Transfers::UniqueTransfer => serialize_i32_as_str(serializer, 1),
            Transfers::TwoTransfers => serialize_i32_as_str(serializer, 2),
            Transfers::Other(a) => serialize_i32_as_str(serializer, *a),
            Transfers::Unlimited => serializer.serialize_none(),
        }
    }
}
/// Defines the type of a [StopTransfer]
#[derive(Debug, Serialize, Deserialize, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub enum TransferType {
    /// Recommended transfer point between routes
    #[serde(rename = "0")]
    #[derivative(Default)]
    Recommended,
    /// Departing vehicle waits for arriving one
    #[serde(rename = "1")]
    Timed,
    /// Transfer requires a minimum amount of time between arrival and departure to ensure a connection.
    #[serde(rename = "2")]
    MinTime,
    /// Transfer is not possible at this location
    #[serde(rename = "3")]
    Impossible,
}

/// Type of pathway between [from_stop] and [to_stop]
#[derive(Debug, Serialize, Deserialize, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub enum PathwayMode {
    /// A walkway
    #[serde(rename = "1")]
    #[derivative(Default)]
    Walkway,
    /// Stairs
    #[serde(rename = "2")]
    Stairs,
    /// Moving sidewalk / travelator
    #[serde(rename = "3")]
    MovingSidewalk,
    /// Escalator
    #[serde(rename = "4")]
    Escalator,
    /// Elevator
    #[serde(rename = "5")]
    Elevator,
    /// A pathway that crosses into an area of the station where a
    /// proof of payment is required (usually via a physical payment gate)
    #[serde(rename = "6")]
    FareGate,
    /// Indicates a pathway exiting an area where proof-of-payment is required
    /// into an area where proof-of-payment is no longer required.
    #[serde(rename = "7")]
    ExitGate,
}

/// Indicates in which direction the pathway can be used
#[derive(Debug, Serialize, Deserialize, Derivative, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub enum PathwayDirectionType {
    /// Unidirectional pathway, it can only be used from [from_stop_id] to [to_stop_id].
    #[serde(rename = "0")]
    #[derivative(Default)]
    Unidirectional,
    /// Bidirectional pathway, it can be used in the two directions.
    #[serde(rename = "1")]
    Bidirectional,
}
