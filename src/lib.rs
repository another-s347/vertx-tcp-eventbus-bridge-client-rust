extern crate bytes;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
extern crate crossbeam;
extern crate tokio;
extern crate tokio_io;
extern crate futures;
pub mod protocol;
pub mod response;
pub mod request;
pub mod eventbus;

#[cfg(test)]
mod tests {
    use std::io::Error;
    use std::net::SocketAddr;
    use eventbus;

    fn test_connect()->Result<(),Error>{
        let mut eb=eventbus::Eventbus::connect("127.0.0.1:12345").unwrap();
        eb.send("test", json!({
            "aaaa":"bbbb"
        }), |r|{
            println!("callback {:?}", r);
            println!("callback {}", 1);
        });
        loop{};
    }

    #[test]
    fn it_works() {
        match test_connect() {
            Ok(_)=>(),
            Err(e)=>println!("{}",e)
        }
    }
}
