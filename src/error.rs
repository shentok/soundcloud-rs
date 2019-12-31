// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error;
use std::fmt;
use std::io;
use std::result;

use hyper;
use serde_json;
use url;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError(String),
    ParseError(url::ParseError),
    JsonError(serde_json::Error),
    HttpError(hyper::Error),
    InvalidFilter(String),
    Io(io::Error),
    UriError(hyper::http::uri::InvalidUri),
    TrackNotDownloadable,
    TrackNotStreamable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::JsonError(ref error) => write!(f, "JSON error: {}", error),
            Error::HttpError(ref error) => write!(f, "HTTP error: {}", error),
            Error::ApiError(ref error) => write!(f, "SoundCloud error: {}", error),
            Error::ParseError(ref error) => write!(f, "Parse error: {}", error),
            Error::Io(ref error) => write!(f, "IO error: {}", error),
            Error::UriError(ref error) => write!(f, "URI error: {}", error),
            Error::InvalidFilter(_) => write!(f, "Invalid filter"),
            Error::TrackNotStreamable => write!(f, "The track is not available for streaming"),
            Error::TrackNotDownloadable => write!(f, "The track is not available for download"),
        }
    }
}

impl PartialEq<Error> for Error {
    fn eq(&self, other: &Self) -> bool {
        match *self {
            Error::JsonError(ref lhs) => {
                if let Error::JsonError(ref rhs) = other {
                    lhs.to_string() == rhs.to_string()
                } else {
                    false
                }
            }
            Error::HttpError(ref lhs) => {
                if let Error::HttpError(ref rhs) = other {
                    lhs.to_string() == rhs.to_string()
                } else {
                    false
                }
            }
            Error::ApiError(ref lhs) => {
                if let Error::ApiError(ref rhs) = other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Error::ParseError(ref lhs) => {
                if let Error::ParseError(ref rhs) = other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Error::Io(_) => {
                if let Error::Io(_) = other {
                    self.to_string() == other.to_string()
                } else {
                    false
                }
            }
            Error::UriError(ref lhs) => {
                if let Error::UriError(ref rhs) = other {
                    lhs.to_string() == rhs.to_string()
                } else {
                    false
                }
            }
            Error::InvalidFilter(ref lhs) => {
                if let Error::InvalidFilter(ref rhs) = other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Error::TrackNotStreamable => {
                if let Error::TrackNotDownloadable = other {
                    true
                } else {
                    false
                }
            }
            Error::TrackNotDownloadable => {
                if let Error::TrackNotDownloadable = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::JsonError(ref error) => Some(error),
            Error::HttpError(ref error) => Some(error),
            Error::Io(ref error) => Some(error),
            _ => None,
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::HttpError(error)
    }
}

impl From<hyper::http::uri::InvalidUri> for Error {
    fn from(error: hyper::http::uri::InvalidUri) -> Error {
        Error::UriError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::JsonError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Error {
        Error::ParseError(error)
    }
}
