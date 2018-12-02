#![feature(await_macro, async_await, futures_api)]
#[macro_use]
extern crate tokio;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
pub mod protocol;
pub mod response;
pub mod request;
pub mod eventbus;

#[cfg(test)]
mod tests {
    use std::io::Error;
    use std::net::SocketAddr;
    use crate::eventbus;

    #[test]
    fn test_async(){
        tokio::run_async(async {
            let mut eb=await!(eventbus::Eventbus::connectFuture("127.0.0.1:12345"));
            await!(eb.send("test".to_string(), json!({
                "aaaa":"bbbb"
            }), |r|{
                println!("callback {:?}", r);
                println!("callback {}", 1);
            }));
        });
    }
}
