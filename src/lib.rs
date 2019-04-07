extern crate byteorder;
extern crate bytes;
extern crate crossbeam;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod protocol;
pub mod response;
pub mod request;
//pub mod eventbus;
pub mod future;
pub mod codec;

#[cfg(test)]
mod tests {
    use std::io::Error;
    use std::net::{SocketAddr, IpAddr};
    use std::str::FromStr;
    use crate::future;
    use futures::future::{IntoFuture,Future};
    use tokio::prelude::stream::Stream;
    use crate::future::Eventbus;

    /// Current Output:
    /// running 1 test
    /// MESSAGE_FAIL(ResponseFailObject { failureCode: 1, failureType: "RECIPIENT_FAILURE", message: "test fail message", sourceAddress: "test" })
    #[test]
    fn test_send() {
        let task = future::Eventbus::connect(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        let task = task.and_then(|(eventbus, readstream, writestream)| {
            tokio::spawn(readstream.into_future().map(|_|()).map_err(|e|()));
            tokio::spawn(writestream.into_future().map(|_|()).map_err(|e|()));
            futures::future::ok(eventbus)
        });
        let task = task.and_then(|eventbus:Eventbus|{
            let test_reply=eventbus.send_reply("test".to_string(),json!({
                "aaaa":"bbbb"
            })).unwrap().and_then(|response|{
                println!("{:?}",response);
                futures::future::ok(())
            });
            test_reply
        });
        tokio::run(task.map_err(|e|()));
    }
}
