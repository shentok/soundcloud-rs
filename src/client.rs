// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use url::Url;
use hyper;
use hyper_tls;
use futures::future;
use futures::future::Either;
use futures::{Future, Stream};
use tokio_core;

use std::borrow::Borrow;
use std::io::{Write};

use track::{Track, TrackRequestBuilder, SingleTrackRequestBuilder};
use error::{Error, Result};

pub type Params<'a, K, V> = &'a [(K, V)];

#[derive(Debug)]
pub struct Client {
    client_id: String,
    http_client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

/// Registered client application.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct App {
    /// Integer ID.
    pub id: usize,
    /// API resource URL.
    pub uri: String,
    /// URL to the SoundCloud.com page
    pub permalink_url: String,
    /// URL to an external site.
    pub external_url: String,
    /// Username of the app creator.
    pub creator: Option<String>,
}

/// User comment.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    /// Integer ID.
    pub id: usize,
    /// API resource URL.
    pub uri: String,
    /// Time of creation, as an unparsed string.
    pub created_at: String,
    /// HTML comment body.
    pub body: String,
    /// Associated timestamp in milliseconds.
    pub timestamp: Option<usize>,
    /// User ID of the commenter.
    pub user_id: usize,
    /// Small representation of the commenters user.
    pub user: User,
    /// The track ID of the related track.
    pub track_id: usize,
}

/// Registered user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// Integer ID.
    pub id: usize,
    /// Permalink of the resource.
    pub permalink: String,
    /// Username.
    pub username: String,
    /// API resource URL.
    pub uri: String,
    /// URL to the SoundCloud.com page.
    pub permalink_url: String,
    /// URL to a JPEG image.
    pub avatar_url: String,
    /// Country.
    pub country: Option<String>,
    /// First and last name.
    pub full_name: Option<String>,
    /// City.
    pub city: Option<String>,
    /// Description, written by the user.
    pub description: Option<String>,
    /// Discogs name.
    #[serde(rename="discogs-name")]
    pub discogs_name: Option<String>, // discogs-name
    /// MySpace name.
    #[serde(rename="myspace-name")]
    pub myspace_name: Option<String>, // myspace-name
    /// URL to a website.
    pub website: Option<String>,
    /// Custom title for the website.
    #[serde(rename="website-title")]
    pub website_title: Option<String>, // website-title
    /// Online status.
    pub online: Option<bool>,
    /// Number of public tracks.
    pub track_count: Option<usize>,
    /// Number of public playlists.
    pub playlist_count: Option<usize>,
    /// Number of followers.
    pub followers_count: Option<usize>,
    /// Number of followed users.
    pub followings_count: Option<usize>,
    /// Number of favorited public tracks.
    pub public_favorites_count: Option<usize>,
    // pub avatar_data …
}

impl Client {
    /// Constructs a new `Client` with the provided `client_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// ```
    pub fn new(client_id: &str, handle: &tokio_core::reactor::Handle) -> Client {
        let client = hyper::Client::configure()
            .connector(hyper_tls::HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);
        // client.set_redirect_policy(hyper::client::RedirectPolicy::FollowNone);

        Client {
            client_id: client_id.to_owned(),
            http_client: client,
        }
    }

    /// Returns the client id.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Creates and sends a HTTP GET request to the API endpoint.
    ///
    /// A `client_id` parameter will automatically be added to the request.
    ///
    /// Returns the HTTP response on success, an error otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// use soundcloud::Client;
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// let response = client.get("/resolve", Some(&[("url",
    /// "https://soundcloud.com/firepowerrecs/afk-shellshock-kamikaze-promo-mix-lock-load-series-vol-20")]));
    ///
    /// let mut buffer = String::new();
    /// response.unwrap().read_to_string(&mut buffer);
    ///
    /// assert!(!buffer.is_empty());
    /// ```
    pub fn get<I, K, V>(&self, path: &str, params: Option<I>)
        -> hyper::client::FutureResponse
    where I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str> {
        let mut url = Url::parse(&format!("https://{}{}", super::API_HOST, path)).unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();

            if let Some(params) = params {
                query_pairs.extend_pairs(params);
            }
        }

        let uri = self.parse_url(url).unwrap();
        self.http_client.get(uri)
    }

    pub fn download<'a, 'b, W: 'a + Write>(&'a self, track: &'b Track, mut writer: W) -> Box<Future<Item=usize, Error=Error> + 'a> {
        if !track.downloadable || !track.download_url.is_some() {
            return Box::new(future::err(Error::TrackNotDownloadable));
        }

        let url = self.parse_url(track.download_url.as_ref().unwrap()).unwrap();

        let response = self.http_client.get(url)
            .and_then(move |response| {
                // Follow the redirect just this once.
                if let Some(header) = response.headers().get::<hyper::header::Location>().cloned() {
                    let uri = header.parse().unwrap();
                    let inner_response = self.http_client.get(uri);
                    Either::A(inner_response)
                }
                else {
                    Either::B(future::ok(response))
                }
            })
            .map(move |response| {
                response.body()
                    .fold(0, move |acc, chunk| {
                        let result = match writer.write(chunk.as_ref()) {
                            Ok(num_written) => Ok(acc + num_written),
                            error => error,
                        };

                        result
                    })
            })
            .flatten()
            .map_err(|error| {
                Error::HttpError(error)
            });

        Box::new(response)
    }

    /// Starts streaming the track provided in the tracks `stream_url` to the `writer` if the track
    /// is streamable via the API.
    pub fn stream<'a, 'b, W: 'a + Write>(&'a self, track: &'b Track, mut writer: W) -> Box<Future<Item=usize, Error=Error> + 'a>  {
        if !track.streamable || !track.stream_url.is_some() {
            return Box::new(future::err(Error::TrackNotStreamable));
        }

        let url = self.parse_url(track.stream_url.as_ref().unwrap()).unwrap();

        let response = self.http_client.get(url)
            .and_then(move |response| {
                // Follow the redirect just this once.
                if let Some(header) = response.headers().get::<hyper::header::Location>().cloned() {
                    let uri = header.parse().unwrap();
                    let inner_response = self.http_client.get(uri);
                    Either::A(inner_response)
                }
                else {
                    Either::B(future::ok(response))
                }
            })
            .map(move |response| {
                response.body()
                    .fold(0, move |acc, chunk| {
                        let result = match writer.write(chunk.as_ref()) {
                            Ok(num_written) => Ok(acc + num_written),
                            error => error,
                        };

                        result
                    })
            })
            .flatten()
            .map_err(|error| {
                Error::HttpError(error)
            });

        Box::new(response)
    }

    /// Resolves any soundcloud resource and returns it as a `Url`.
    pub fn resolve(&self, url: &str) -> Box<Future<Item=Url, Error=Error>> {
        let uri = self.get("/resolve", Some(&[("url", url)]));

        let response = uri
            .map_err(|error| { Error::HttpError(error) })
            .and_then(|response| {
                match response.headers().get::<hyper::header::Location>() {
                    Some(header) => {
                        future::ok(Url::parse(header.as_ref()).unwrap())
                    },
                    _ => {
                        future::err(Error::ApiError("expected location header".to_owned()))
                    }
                }
            });

        Box::new(response)
    }

    /// Returns a builder for a single track-by-id request.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// let track = client.track(262681089).get();
    ///
    /// assert_eq!(track.unwrap().id, 262681089);
    /// ```
    pub fn track(&self, id: usize) -> SingleTrackRequestBuilder {
        SingleTrackRequestBuilder::new(self, id)
    }

    /// Returns a builder for searching tracks with multiple criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// use soundcloud::Client;
    ///
    /// let client = Client::new(env!("SOUNDCLOUD_CLIENT_ID"));
    /// let tracks = client.tracks().genres(Some(["HipHop"])).get();
    ///
    /// assert!(tracks.unwrap().expect("no tracks found").len() > 0);
    /// ```
    pub fn tracks(&self) -> TrackRequestBuilder {
        TrackRequestBuilder::new(self)
    }

    /// Parses a string and returns a url with the client_id query parameter set.
    pub fn parse_url<S: AsRef<str>>(&self, url: S) -> Result<hyper::Uri> {
        let mut url = Url::parse(url.as_ref()).unwrap();
        url.query_pairs_mut().append_pair("client_id", &self.client_id);
        url.as_str().parse().map_err(|error| { Error::UriError(error) } )
    }
}

#[cfg(test)]
mod tests {
    use url::Url;
    use super::*;

    fn client(handle: &tokio_core::reactor::Handle) -> Client {
        Client::new(env!("SOUNDCLOUD_CLIENT_ID"), handle)
    }

    #[test]
    fn test_resolve_track() {
        let mut core = tokio_core::reactor::Core::new().unwrap();

        let work = client(&core.handle()).resolve("https://soundcloud.com/isqa/tree-eater-1");

        let result = core.run(work);

        assert_eq!(result.unwrap(),
            Url::parse(&format!("https://api.soundcloud.com/tracks/262976655?client_id={}",
                                env!("SOUNDCLOUD_CLIENT_ID"))).unwrap());
    }

    #[test]
    fn test_get_tracks() {
        let mut core = tokio_core::reactor::Core::new().unwrap();

        let work = client(&core.handle()).tracks().query(Some("d0df0dt snuffx")).get();

        let result = core.run(work);

        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_get_track() {
        let mut core = tokio_core::reactor::Core::new().unwrap();

        let work = client(&core.handle()).tracks().id(18201932).get();

        let track = core.run(work).unwrap();

        assert_eq!(track.id, 18201932);
    }

    #[test]
    fn test_download_track() {
        use std::fs;
        use std::path::Path;

        let mut core = tokio_core::reactor::Core::new().unwrap();

        let client = client(&core.handle());
        let path = Path::new("hi.mp3");
        let mut file = fs::File::create(path).unwrap();
        let work = client.tracks().id(263801976).get()
            .and_then(|track| client.download(&track, &mut file) );

        let ret = core.run(work);

        assert!(ret.unwrap() > 0);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_stream_track() {
        use std::io::BufWriter;

        let mut buffer = BufWriter::new(vec![]);
        {
            let mut core = tokio_core::reactor::Core::new().unwrap();

            let client = client(&core.handle());
            let work = client.tracks().id(262681089).get()
                .and_then(|track| client.stream(&track, &mut buffer));

            let len = core.run(work);

            assert!(len.unwrap() > 0);
        }
        buffer.flush();
        assert!(buffer.get_ref().len() > 0);
    }
}
