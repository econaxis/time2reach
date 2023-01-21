/*! The [General Transit Feed Specification](https://gtfs.org/) (GTFS) is a commonly used model to represent public transit data.

This crates brings [serde](https://serde.rs) structures of this model and helpers to read GTFS files.

To get started, see [Gtfs].

## What is GTFS

A Gtfs feed is a collection of CSV files (often bundled as a zip file).
Each file represents a collection of one type (stops, lines, etc.) that have relationships through unique identifiers.

This crate reads a feed, deserializes the objects into Rust structs and verifies the relationships.

## Design decisions

### Two representations

The [RawGtfs] representation holds the objects as close as possible to their CSV representation. This allows to check invalid references.

[Gtfs] re-organizes a bit the data. For instance all the [StopTime] are included within their corresponding [Trip] and cannot be accessed directly.
If an object references a non existing Id it will be an error.

### Use of Enum

Many values are integers that are actually enumerations of certain values. We always use Rust enums, like [LocationType] to represent them, and not the integer value.

### Reference

We try to stick as closely as possible to the reference. Optional fields are [std::option], while missing mandatory elements will result in an error.
If a default value is defined, we will use it.

There are two references <https://gtfs.org/reference/static> and <https://developers.google.com/transit/gtfs/reference>. They are mostly the same, even if google’s specification has some extensions.

### Renaming

We kept some names even if they can be confusing (a [Calendar] will be referenced by `service_id`), but we strip the object type (`route_short_name` is [Route::short_name]).

*/
#![warn(missing_docs)]

#[macro_use]
extern crate derivative;
#[macro_use]
extern crate serde_derive;

mod enums;
pub mod error;
mod gtfs;
mod gtfs_reader;
pub(crate) mod objects;
mod raw_gtfs;
mod serde_helpers;

#[cfg(test)]
mod tests;

pub use error::Error;
pub use gtfs::Gtfs;
pub use gtfs_reader::GtfsReader;
pub use objects::*;
pub use raw_gtfs::RawGtfs;
