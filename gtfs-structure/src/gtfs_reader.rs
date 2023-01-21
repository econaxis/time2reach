use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{Error, Gtfs, RawGtfs};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Allows to parameterize how the parsing library behaves
///
/// ```
///let gtfs = gtfs_structures::GtfsReader::default()
///    .read_stop_times(false) // Won’t read the stop times to save time and memory
///    .unkown_enum_as_default(false) // Won’t convert unknown enumerations into default (e.g. LocationType=42 considered as a stop point)
///    .read("fixtures/zips/gtfs.zip")?;
///assert_eq!(0, gtfs.trips.get("trip1").unwrap().stop_times.len());
/// # Ok::<(), gtfs_structures::error::Error>(())
///```
///
/// You can also get a [RawGtfs] by doing
/// ```
///let gtfs = gtfs_structures::GtfsReader::default()
///    .read_stop_times(false)
///    .raw()
///    .read("fixtures/zips/gtfs.zip")?;
///assert_eq!(1, gtfs.trips?.len());
///assert_eq!(0, gtfs.stop_times?.len());
/// # Ok::<(), gtfs_structures::error::Error>(())
///```
#[derive(Derivative)]
#[derivative(Default)]
pub struct GtfsReader {
    /// [crate::objects::StopTime] are very large and not always needed. This allows to skip reading them
    #[derivative(Default(value = "true"))]
    pub read_stop_times: bool,
    /// If a an enumeration has un unknown value, should we use the default value
    #[derivative(Default(value = "false"))]
    pub unkown_enum_as_default: bool,
    /// Avoid trimming the fields
    ///
    /// It is quite time consumming
    /// If performance is an issue, and if your data is high quality, you can switch it off
    #[derivative(Default(value = "true"))]
    pub trim_fields: bool,
}

impl GtfsReader {
    /// Configures the reader to read or not the stop times (default: true)
    ///
    /// This can be useful to save time and memory with large datasets when the timetable are not needed
    /// Returns Self and can be chained
    pub fn read_stop_times(mut self, read_stop_times: bool) -> Self {
        self.read_stop_times = read_stop_times;
        self
    }

    /// If a an enumeration has un unknown value, should we use the default value (default: false)
    ///
    /// For instance, if [crate::objects::Stop] has a [crate::objects::LocationType] with a value 42 in the GTFS
    /// when true, we will parse it as StopPoint
    /// when false, we will parse it as Unknown(42)
    /// Returns Self and can be chained
    pub fn unkown_enum_as_default(mut self, unkown_enum_as_default: bool) -> Self {
        self.unkown_enum_as_default = unkown_enum_as_default;
        self
    }

    /// Should the fields be trimmed (default: true)
    ///
    /// It is quite time consumming
    /// If performance is an issue, and if your data is high quality, you can set it to false
    pub fn trim_fields(mut self, trim_fields: bool) -> Self {
        self.trim_fields = trim_fields;
        self
    }

    /// Reads from an url (if starts with `"http"`), or a local path (either a directory or zipped file)
    ///
    /// To read from an url, build with read-url feature
    /// See also [Gtfs::from_url] and [Gtfs::from_path] if you don’t want the library to guess
    pub fn read(self, gtfs: &str) -> Result<Gtfs, Error> {
        self.raw().read(gtfs).and_then(Gtfs::try_from)
    }

    /// Reads the raw GTFS from a local zip archive or local directory
    pub fn read_from_path<P>(self, path: P) -> Result<Gtfs, Error>
    where
        P: AsRef<Path> + std::fmt::Display,
    {
        self.raw().read_from_path(path).and_then(Gtfs::try_from)
    }

    /// Reads the GTFS from a remote url
    ///
    /// The library must be built with the read-url feature
    #[cfg(feature = "read-url")]
    pub fn read_from_url<U: reqwest::IntoUrl>(self, url: U) -> Result<Gtfs, Error> {
        self.raw().read_from_url(url).and_then(Gtfs::try_from)
    }

    /// Asynchronously reads the GTFS from a remote url
    ///
    /// The library must be built with the read-url feature
    #[cfg(feature = "read-url")]
    pub async fn read_from_url_async<U: reqwest::IntoUrl>(self, url: U) -> Result<Gtfs, Error> {
        self.raw()
            .read_from_url_async(url)
            .await
            .and_then(Gtfs::try_from)
    }

    /// Read the Gtfs as a [RawGtfs].
    ///
    /// ```
    ///let gtfs = gtfs_structures::GtfsReader::default()
    ///    .read_stop_times(false)
    ///    .raw()
    ///    .read("fixtures/zips/gtfs.zip")?;
    ///assert_eq!(1, gtfs.trips?.len());
    ///assert_eq!(0, gtfs.stop_times?.len());
    /// # Ok::<(), gtfs_structures::error::Error>(())
    ///```
    pub fn raw(self) -> RawGtfsReader {
        RawGtfsReader { reader: self }
    }
}

/// This reader generates [RawGtfs]. It must be built using [GtfsReader::raw]
///
/// The methods to read a Gtfs are the same as for [GtfsReader]
pub struct RawGtfsReader {
    reader: GtfsReader,
}

impl RawGtfsReader {
    fn read_from_directory(&self, p: &std::path::Path) -> Result<RawGtfs, Error> {
        let now = Utc::now();
        // Thoses files are not mandatory
        // We use None if they don’t exist, not an Error
        let files = std::fs::read_dir(p)?
            .filter_map(|d| d.ok().and_then(|p| p.path().to_str().map(|s| s.to_owned())))
            .collect();

        let mut result = RawGtfs {
            trips: self.read_objs_from_path(p.join("trips.txt")),
            calendar: self.read_objs_from_optional_path(p, "calendar.txt"),
            calendar_dates: self.read_objs_from_optional_path(p, "calendar_dates.txt"),
            stops: self.read_objs_from_path(p.join("stops.txt")),
            routes: self.read_objs_from_path(p.join("routes.txt")),
            stop_times: if self.reader.read_stop_times {
                self.read_objs_from_path(p.join("stop_times.txt"))
            } else {
                Ok(Vec::new())
            },
            agencies: self.read_objs_from_path(p.join("agency.txt")),
            shapes: self.read_objs_from_optional_path(p, "shapes.txt"),
            fare_attributes: self.read_objs_from_optional_path(p, "fare_attributes.txt"),
            frequencies: self.read_objs_from_optional_path(p, "frequencies.txt"),
            transfers: self.read_objs_from_optional_path(p, "transfers.txt"),
            pathways: self.read_objs_from_optional_path(p, "pathways.txt"),
            feed_info: self.read_objs_from_optional_path(p, "feed_info.txt"),
            read_duration: Utc::now().signed_duration_since(now).num_milliseconds(),
            files,
            sha256: None,
        };

        if self.reader.unkown_enum_as_default {
            result.unknown_to_default();
        }
        Ok(result)
    }

    /// Reads from an url (if starts with `"http"`) if the feature `read-url` is activated,
    /// or a local path (either a directory or zipped file)
    pub fn read(self, gtfs: &str) -> Result<RawGtfs, Error> {
        #[cfg(feature = "read-url")]
        if gtfs.starts_with("http") {
            return self.read_from_url(gtfs);
        }
        self.read_from_path(gtfs)
    }

    /// Reads the GTFS from a remote url
    #[cfg(feature = "read-url")]
    pub fn read_from_url<U: reqwest::IntoUrl>(self, url: U) -> Result<RawGtfs, Error> {
        let mut res = reqwest::blocking::get(url)?;
        let mut body = Vec::new();
        res.read_to_end(&mut body)?;
        let cursor = std::io::Cursor::new(body);
        self.read_from_reader(cursor)
    }

    /// Asynchronously reads the GTFS from a remote url
    #[cfg(feature = "read-url")]
    pub async fn read_from_url_async<U: reqwest::IntoUrl>(self, url: U) -> Result<RawGtfs, Error> {
        let res = reqwest::get(url).await?.bytes().await?;
        let reader = std::io::Cursor::new(res);
        self.read_from_reader(reader)
    }

    /// Reads the raw GTFS from a local zip archive or local directory
    pub fn read_from_path<P>(&self, path: P) -> Result<RawGtfs, Error>
    where
        P: AsRef<Path> + std::fmt::Display,
    {
        let p = path.as_ref();
        if p.is_file() {
            let reader = File::open(p)?;
            self.read_from_reader(reader)
        } else if p.is_dir() {
            self.read_from_directory(p)
        } else {
            Err(Error::NotFileNorDirectory(format!("{}", p.display())))
        }
    }

    pub fn read_from_reader<T: std::io::Read + std::io::Seek>(
        &self,
        reader: T,
    ) -> Result<RawGtfs, Error> {
        let now = Utc::now();
        let mut hasher = Sha256::new();
        let mut buf_reader = std::io::BufReader::new(reader);
        let _n = std::io::copy(&mut buf_reader, &mut hasher)?;
        let hash = hasher.finalize();
        let mut archive = zip::ZipArchive::new(buf_reader)?;
        let mut file_mapping = HashMap::new();
        let mut files = Vec::new();

        for i in 0..archive.len() {
            let archive_file = archive.by_index(i)?;
            files.push(archive_file.name().to_owned());

            for gtfs_file in &[
                "agency.txt",
                "calendar.txt",
                "calendar_dates.txt",
                "routes.txt",
                "stops.txt",
                "stop_times.txt",
                "trips.txt",
                "fare_attributes.txt",
                "frequencies.txt",
                "transfers.txt",
                "pathways.txt",
                "feed_info.txt",
                "shapes.txt",
            ] {
                let path = std::path::Path::new(archive_file.name());
                if path.file_name() == Some(std::ffi::OsStr::new(gtfs_file)) {
                    file_mapping.insert(gtfs_file, i);
                    break;
                }
            }
        }

        let mut result = RawGtfs {
            agencies: self.read_file(&file_mapping, &mut archive, "agency.txt"),
            calendar: self.read_optional_file(&file_mapping, &mut archive, "calendar.txt"),
            calendar_dates: self.read_optional_file(
                &file_mapping,
                &mut archive,
                "calendar_dates.txt",
            ),
            routes: self.read_file(&file_mapping, &mut archive, "routes.txt"),
            stops: self.read_file(&file_mapping, &mut archive, "stops.txt"),
            stop_times: if self.reader.read_stop_times {
                self.read_file(&file_mapping, &mut archive, "stop_times.txt")
            } else {
                Ok(Vec::new())
            },
            trips: self.read_file(&file_mapping, &mut archive, "trips.txt"),
            fare_attributes: self.read_optional_file(
                &file_mapping,
                &mut archive,
                "fare_attributes.txt",
            ),
            frequencies: self.read_optional_file(&file_mapping, &mut archive, "frequencies.txt"),
            transfers: self.read_optional_file(&file_mapping, &mut archive, "transfers.txt"),
            pathways: self.read_optional_file(&file_mapping, &mut archive, "pathways.txt"),
            feed_info: self.read_optional_file(&file_mapping, &mut archive, "feed_info.txt"),
            shapes: self.read_optional_file(&file_mapping, &mut archive, "shapes.txt"),
            read_duration: Utc::now().signed_duration_since(now).num_milliseconds(),
            files,
            sha256: Some(format!("{:x}", hash)),
        };

        if self.reader.unkown_enum_as_default {
            result.unknown_to_default();
        }
        Ok(result)
    }

    fn read_objs<T, O>(&self, mut reader: T, file_name: &str) -> Result<Vec<O>, Error>
    where
        for<'de> O: Deserialize<'de>,
        T: std::io::Read,
    {
        let mut bom = [0; 3];
        reader
            .read_exact(&mut bom)
            .map_err(|e| Error::NamedFileIO {
                file_name: file_name.to_owned(),
                source: Box::new(e),
            })?;

        let chained = if bom != [0xefu8, 0xbbu8, 0xbfu8] {
            bom.chain(reader)
        } else {
            [].chain(reader)
        };

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(if self.reader.trim_fields {
                csv::Trim::Fields
            } else {
                csv::Trim::None
            })
            .from_reader(chained);
        // We store the headers to be able to return them in case of errors
        let headers = reader
            .headers()
            .map_err(|e| Error::CSVError {
                file_name: file_name.to_owned(),
                source: e,
                line_in_error: None,
            })?
            .clone();

        // Pre-allocate a StringRecord for performance reasons
        let mut rec = csv::StringRecord::new();
        let mut objs = Vec::new();

        // Read each record into the pre-allocated StringRecord one at a time
        while reader.read_record(&mut rec).map_err(|e| Error::CSVError {
            file_name: file_name.to_owned(),
            source: e,
            line_in_error: None,
        })? {
            let obj = rec
                .deserialize(Some(&headers))
                .map_err(|e| Error::CSVError {
                    file_name: file_name.to_owned(),
                    source: e,
                    line_in_error: Some(crate::error::LineError {
                        headers: headers.into_iter().map(String::from).collect(),
                        values: rec.into_iter().map(String::from).collect(),
                    }),
                })?;
            objs.push(obj);
        }
        Ok(objs)
    }

    fn read_objs_from_path<O>(&self, path: std::path::PathBuf) -> Result<Vec<O>, Error>
    where
        for<'de> O: Deserialize<'de>,
    {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("invalid_file_name")
            .to_string();
        if path.exists() {
            File::open(path)
                .map_err(|e| Error::NamedFileIO {
                    file_name: file_name.to_owned(),
                    source: Box::new(e),
                })
                .and_then(|r| self.read_objs(r, &file_name))
        } else {
            Err(Error::MissingFile(file_name))
        }
    }

    fn read_objs_from_optional_path<O>(
        &self,
        dir_path: &std::path::Path,
        file_name: &str,
    ) -> Option<Result<Vec<O>, Error>>
    where
        for<'de> O: Deserialize<'de>,
    {
        File::open(dir_path.join(file_name))
            .ok()
            .map(|r| self.read_objs(r, file_name))
    }

    fn read_file<O, T>(
        &self,
        file_mapping: &HashMap<&&str, usize>,
        archive: &mut zip::ZipArchive<T>,
        file_name: &str,
    ) -> Result<Vec<O>, Error>
    where
        for<'de> O: Deserialize<'de>,
        T: std::io::Read + std::io::Seek,
    {
        self.read_optional_file(file_mapping, archive, file_name)
            .unwrap_or_else(|| Err(Error::MissingFile(file_name.to_owned())))
    }

    fn read_optional_file<O, T>(
        &self,
        file_mapping: &HashMap<&&str, usize>,
        archive: &mut zip::ZipArchive<T>,
        file_name: &str,
    ) -> Option<Result<Vec<O>, Error>>
    where
        for<'de> O: Deserialize<'de>,
        T: std::io::Read + std::io::Seek,
    {
        file_mapping.get(&file_name).map(|i| {
            self.read_objs(
                archive.by_index(*i).map_err(|e| Error::NamedFileIO {
                    file_name: file_name.to_owned(),
                    source: Box::new(e),
                })?,
                file_name,
            )
        })
    }
}
