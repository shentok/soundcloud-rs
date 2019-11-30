// Copyright (c) 2016, Mikkel Kroman <mk@uplink.io>
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// * Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
//
// * Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

extern crate soundcloud;
extern crate tokio_core;
extern crate futures;

use futures::future::Future;
use std::io::Write;

fn main() {
    let soundcloud_client_id = std::env::var("SOUNDCLOUD_CLIENT_ID").expect("SOUNDCLOUD_CLIENT_ID");
    let mut core = tokio_core::reactor::Core::new().unwrap();

    let client = soundcloud::Client::new(&soundcloud_client_id);
    let work = client.tracks().query(Some("noisia")).get()
        .and_then(|tracks| {
            match tracks {
                Some(tracks) => tracks.iter().for_each(|track| { std::io::stdout().write_all(format!("{}\n", track.title).as_ref()); } ),
                None => { std::io::stdout().write("no tracks found".as_ref()).unwrap(); () }
            };

            Ok(())
        });

    core.run(work).unwrap();
}
