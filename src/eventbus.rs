use super::protocol;
use std::io::Cursor;
use std::io::Read;
use tokio::net::TcpStream;
use tokio::prelude::*;
use std::net::SocketAddr;
use std::io::Write;
use std::io::Error;
use std::ops::Deref;
use serde_json::Value;
use std::collections::HashMap;
use super::request;
use std::thread;
use std::sync::{Mutex, Arc};
use byteorder::{BigEndian, ReadBytesExt};
use std::ops::DerefMut;
use super::response;
use std::str;
use crossbeam;
use tokio::run;

type handlerBox = Box<Fn(response::Response) + Send + Sync>;

pub struct Eventbus {
    client: TcpStream,
    handlers_access: Arc<Mutex<HashMap<String, Box<Fn(response::Response) + Send + 'static + Sync>>>>,
}

impl Eventbus {
    pub async fn send_frame(&mut self, message: String) {
        let p = super::protocol::Protocol {
            message_length: message.len() as i32,
            json_string: message,
        };
        let p_bytes = p.to_bytes();
        await!(self.client.write_all_async(p_bytes.deref())).unwrap();
    }

    pub async fn connectFuture(addr: &str) -> Eventbus {
        let mut tcpstream = await!(TcpStream::connect(&addr.parse::<SocketAddr>().unwrap())).unwrap();
        let counter: Arc<Mutex<HashMap<String, Box<Fn(response::Response) + Send + 'static + Sync>>>> = Arc::new(Mutex::new(HashMap::new()));
        let c_clone = counter.clone();
        let mut length_bytes = vec![0u8; 4];
        loop {
            await!(tcpstream.read_exact_async(&mut length_bytes)).unwrap();
            let mut message_length = Cursor::new(length_bytes.clone());
            let l: i32 = message_length.read_i32::<BigEndian>().unwrap();
            let l_usize = l as usize;
            if (l_usize as i32) < l {
                panic!("packet is too big");
            }
            let mut message_bytes = vec![0u8; l_usize];
            await!(tcpstream.read_exact_async(&mut message_bytes)).unwrap();
            let s = str::from_utf8(&message_bytes).unwrap();
            let (res, add) = response::Response::from_str(s);
            {
                let n = c_clone.lock().unwrap();
                let v = &*n.get(&add).unwrap();
                crossbeam::scope(|scope| {
                    scope.spawn(|| {
                        v(res);
                    });
                });
            }
        }
        Eventbus {
            client: tcpstream,
            handlers_access: counter,
        }
    }

    pub async fn send<T>(&mut self, address: String, message: Value, handler: T)
        where T: Fn(response::Response) + 'static + Send + Sync
    {
        let replyAddress=address.clone();
        let key=address.clone();
        let req = request::Request::send(request::RegularRequestObject {
            address: address,
            body: message,
            headers: None,
            replyAddress: Some(replyAddress),
        });
        {
            let mut p = self.handlers_access.lock().unwrap();
            (*p).insert(key, Box::new(handler));
        }
        let s = req.to_json().to_string();
       await!(self.send_frame(s))
    }

    pub async fn publish(&mut self, address: String, message: Value) {
        let req = request::Request::publish(request::RegularRequestObject {
            address: address,
            body: message,
            headers: None,
            replyAddress: None,
        });
        let s = req.to_json().to_string();
        await!(self.send_frame(s))
    }

    pub async fn register(&mut self, address: String, headers: Option<Value>) {
        let req = request::Request::register(request::RegisterObject {
            address: address,
            headers,
        });
        let s = req.to_json().to_string();
        await!(self.send_frame(s))
    }

    pub async fn unregister(&mut self, address: String, headers: Option<Value>) {
        let req = request::Request::register(request::RegisterObject {
            address: address,
            headers,
        });
        let s = req.to_json().to_string();
        await!(self.send_frame(s))
    }

    pub async fn ping(&mut self) {
        let s = request::Request::ping.to_json().to_string();
        await!(self.send_frame(s))
    }
}
