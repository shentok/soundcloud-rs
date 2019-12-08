// Copyright 2016 Mikkel Kroman <mk@uplink.io>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
//! SoundCloud API library
//!
//! This soundcloud library provides an interface where you can query soundcloud for information
//! about tracks and users.
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate log;
extern crate serde;
extern crate url;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

/// The static host address for the API.
pub const API_HOST: &'static str = "api.soundcloud.com";

mod client;
pub mod error;
mod track;

// Re-export commonly used resources.
pub use client::Client;
pub use client::{App, Comment, User};
pub use error::Error;
pub use track::Track;
