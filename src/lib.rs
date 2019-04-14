//! # A future based vert.x tcp eventbus client implementation for Rust.
//! `Eventbus` is the core struct to communicate with [vert.x](https://vertx.io/) eventbus.
//! Any further operation can be represented by a stream or future.
//! The body of a message is a json value [serde_json::value](https://docs.serde.rs/serde_json/value/enum.Value.html)
//! # Example
//! ```
//! let task = future::Eventbus::connect(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
//! let task = task.and_then(|(eventbus, readstream, writestream)| {
//!     tokio::spawn(readstream.into_future().map(|_| ()).map_err(|e| ()));
//!     tokio::spawn(writestream.into_future().map(|_| ()).map_err(|e| ()));
//!     futures::future::ok(eventbus)
//! });
//! let task = task.and_then(|eventbus: Eventbus| {
//!     let test_reply = eventbus.send_reply("test".to_string(), json!({
//!         "aaaa":"bbbb"
//!     })).unwrap().and_then(|response| {
//!         println!("{:?}", response);
//!         futures::future::ok(())
//!     });
//!     test_reply
//! });
//! tokio::run(task.map_err(|e| ()));
//! ```
extern crate byteorder;
extern crate bytes;
extern crate crossbeam;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod response;
pub mod request;
pub mod future;
pub mod codec;

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::str::FromStr;

    use futures::future::{Future, IntoFuture};
    use tokio::prelude::stream::Stream;

    use crate::future;
    use crate::future::Eventbus;

    /// Current Output:
        /// running 1 test
        /// MessageFail(ResponseFailObject { failureCode: 1, failureType: "RECIPIENT_FAILURE", message: "test fail message", sourceAddress: "test" })
    #[test]
    fn test_send() {
        let task = future::Eventbus::connect(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        let task = task.and_then(|(eventbus, readstream, writestream)| {
            tokio::spawn(readstream.into_future().map(|_| ()).map_err(|e| ()));
            tokio::spawn(writestream.into_future().map(|_| ()).map_err(|e| ()));
            futures::future::ok(eventbus)
        });
        let task = task.and_then(|eventbus: Eventbus| {
            let test_reply = eventbus.send_reply("test".to_string(), json!({
                "aaaa":"bbbb"
            })).unwrap().and_then(|response| {
                println!("{:?}", response);
                futures::future::ok(())
            });
            test_reply
        });
        tokio::run(task.map_err(|e| ()));
    }
}
