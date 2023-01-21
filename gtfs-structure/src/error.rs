//! Module for the error management
use thiserror::Error;

/// Specific line from a CSV file that could not be read
#[derive(Debug)]
pub struct LineError {
    /// Headers of the CSV file
    pub headers: Vec<String>,
    /// Values of the line that could not be parsed
    pub values: Vec<String>,
}

/// An error that can occur when processing GTFS data.
#[derive(Error, Debug)]
pub enum Error {
    /// A mandatory file is not present in the archive
    #[error("Cound not find file {0}")]
    MissingFile(String),
    /// A file references an Id that is not present
    #[error("The id {0} is not known")]
    ReferenceError(String),
    /// The given path to the GTFS is neither a file nor a directory
    #[error("Could not read GTFS: {0} is neither a file nor a directory")]
    NotFileNorDirectory(String),
    /// The time is not given in the HH:MM:SS format
    #[error("'{0}' is not a valid time; HH:MM:SS format is expected.")]
    InvalidTime(String),
    /// The color is not given in the RRGGBB format, without a leading `#`
    #[error("'{0}' is not a valid color; RRGGBB format is expected, without a leading `#`")]
    InvalidColor(String),
    /// Generic Input/Output error while reading a file
    #[error("impossible to read file")]
    IO(#[from] std::io::Error),
    /// Impossible to read a file
    #[error("impossible to read '{file_name}'")]
    NamedFileIO {
        /// The file name that could not be read
        file_name: String,
        /// The inital error that caused the unability to read the file
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Impossible to fetch the remote archive by the URL
    #[cfg(feature = "read-url")]
    #[error("impossible to remotely access file")]
    Fetch(#[from] reqwest::Error),
    /// Impossible to read a CSV file
    #[error("impossible to read csv file '{file_name}'")]
    CSVError {
        /// File name that could not be parsed as CSV
        file_name: String,
        /// The initial error by the csv library
        #[source]
        source: csv::Error,
        /// The line that could not be parsed by the csv library
        line_in_error: Option<LineError>,
    },
    /// Error when trying to unzip the GTFS archive
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
}
