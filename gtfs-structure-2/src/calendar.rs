use rkyv::{Archive, Deserialize, Serialize};

use rustc_hash::FxHashMap;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::gtfs_wrapper::{try_parse_id, vec_to_hashmap, FromWithAgencyId};
use crate::IdType;
use gtfs_structures::CalendarDate;

#[derive(Archive, Serialize, Deserialize, Debug)]
#[archive(check_bytes)]
pub struct NaiveDate1(u32);

#[derive(Archive, Serialize, Deserialize, Debug)]
#[archive(check_bytes)]
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
    pub start_date: NaiveDate1,
    pub end_date: NaiveDate1,
}

impl FromWithAgencyId<gtfs_structures::Calendar> for Service {
    fn from_with_agency_id(agency_id: u16, f: gtfs_structures::Calendar) -> Self
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
            start_date: NaiveDate1(f.start_date.ordinal0()),
            end_date: NaiveDate1(f.end_date.ordinal0()),
        }
    }
}

impl Service {
    #[inline]
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
#[archive(check_bytes)]
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
#[archive(check_bytes)]
pub struct CalendarException {
    /// Identifier of the service that is modified at this date
    pub service_id: IdType,
    /// Date where the service will be added or deleted
    pub date: NaiveDate1,
    /// Is the service added or deleted
    pub exception_type: Exception,
}

impl FromWithAgencyId<gtfs_structures::CalendarDate> for CalendarException {
    fn from_with_agency_id(agency_id: u16, f: CalendarDate) -> Self
    where
        Self: Sized,
    {
        Self {
            service_id: (agency_id, try_parse_id(&f.service_id)),
            date: NaiveDate1(f.date.ordinal0()),
            exception_type: f.exception_type.into(),
        }
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct CalendarExceptionList(FxHashMap<u32, CalendarException>);

impl CalendarExceptionList {
    #[inline]
    fn runs_on_date(&self, date: NaiveDate) -> Option<bool> {
        let ord = date.ordinal0();
        self.0
            .get(&ord)
            .map(|exc| exc.exception_type == Exception::Added)
    }
}

#[derive(Debug, Default, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Calendar {
    pub services: FxHashMap<IdType, Service>,
    pub exceptions: FxHashMap<IdType, CalendarExceptionList>,
}

impl Calendar {
    pub fn extend(&mut self, other: Calendar) {
        self.services.extend(other.services);
        self.exceptions.extend(other.exceptions);
    }
    pub fn runs_on_date(&self, service_id: IdType, date: NaiveDate) -> bool {
        let normal = self.services.get(&service_id).map(|a| a.runs_on_date(date));

        normal.or_else(|| {
            self.exceptions
                .get(&service_id)
                .and_then(|a| a.runs_on_date(date))
        });

        normal.unwrap_or(true)
    }

    pub fn parse(calendar: Vec<Service>, exceptions_list: Vec<CalendarException>) -> Self {
        let services = vec_to_hashmap(calendar, |a| a.id);

        let mut exceptions: FxHashMap<IdType, CalendarExceptionList> = FxHashMap::default();

        for exc in exceptions_list {
            if let Some(inner) = exceptions.get_mut(&exc.service_id) {
                inner.0.insert(exc.date.0, exc);
            } else {
                let service_id = exc.service_id;
                let cal_exception_list = FxHashMap::from_iter([(exc.date.0, exc)]);
                exceptions.insert(service_id, CalendarExceptionList(cal_exception_list));
            }
        }
        Self {
            services,
            exceptions,
        }
    }
}
