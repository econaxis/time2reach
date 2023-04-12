use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::gtfs_wrapper::{try_parse_id, vec_to_hashmap, FromWithAgencyId};
use crate::IdType;
use gtfs_structures::CalendarDate;

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct Service {
    pub id: IdType,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,

    // Date of the form YYYYMMDD, so strcmp corresponds to date compare
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl FromWithAgencyId<gtfs_structures::Calendar> for Service {
    fn from_with_agency_id(agency_id: u8, f: gtfs_structures::Calendar) -> Self
    where
        Self: Sized,
    {
        Service {
            id: (agency_id, try_parse_id(&f.id)),
            monday: f.monday,
            tuesday: f.tuesday,
            wednesday: f.wednesday,
            thursday: f.thursday,
            friday: f.friday,
            saturday: f.saturday,
            sunday: f.sunday,
            start_date: f.start_date,
            end_date: f.end_date,
        }
    }
}

impl Service {
    pub fn runs_on_date(&self, date: NaiveDate) -> bool {
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

#[derive(Serialize, Deserialize, Archive, PartialEq, Eq, Debug)]
pub enum Exception {
    Added,
    Deleted,
}

impl From<gtfs_structures::Exception> for Exception {
    fn from(value: gtfs_structures::Exception) -> Self {
        match value {
            gtfs_structures::Exception::Added => Self::Added,
            gtfs_structures::Exception::Deleted => Self::Deleted,
        }
    }
}

#[derive(Serialize, Deserialize, Archive, Debug)]
pub struct CalendarException {
    /// Identifier of the service that is modified at this date
    pub service_id: IdType,
    /// Date where the service will be added or deleted
    pub date: NaiveDate,
    /// Is the service added or deleted
    pub exception_type: Exception,
}

impl FromWithAgencyId<gtfs_structures::CalendarDate> for CalendarException {
    fn from_with_agency_id(agency_id: u8, f: CalendarDate) -> Self
    where
        Self: Sized,
    {
        Self {
            service_id: (agency_id, try_parse_id(&f.service_id)),
            date: f.date,
            exception_type: f.exception_type.into(),
        }
    }
}

#[derive(Debug)]
pub struct CalendarExceptionList(HashMap<NaiveDate, CalendarException>);

impl CalendarExceptionList {
    fn runs_on_date(&self, date: NaiveDate) -> bool {
        self.0
            .get(&date)
            .map(|exc| exc.exception_type == Exception::Added)
            .unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct Calendar {
    pub services: HashMap<IdType, Service>,
    pub exceptions: HashMap<IdType, CalendarExceptionList>,
}

impl Calendar {
    pub fn runs_on_date(&self, service_id: IdType, date: NaiveDate) -> bool {
        let normal = self.services.get(&service_id).map(|a| a.runs_on_date(date));
        let exception = self
            .exceptions
            .get(&service_id)
            .map(|a| a.runs_on_date(date));

        match (normal, exception) {
            (None, None) => true,
            _ => normal.unwrap_or(false) || exception.unwrap_or(false),
        }
    }

    pub fn parse(calendar: Vec<Service>, exceptions_list: Vec<CalendarException>) -> Self {
        let services = vec_to_hashmap(calendar, |a| a.id);

        let mut exceptions: HashMap<IdType, CalendarExceptionList> = HashMap::new();

        for exc in exceptions_list {
            if let Some(inner) = exceptions.get_mut(&exc.service_id) {
                inner.0.insert(exc.date, exc);
            } else {
                exceptions.insert(
                    exc.service_id,
                    CalendarExceptionList([(exc.date, exc)].into()),
                );
            }
        }
        Self {
            services,
            exceptions,
        }
    }
}
