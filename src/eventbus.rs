use super::protocol;
use std::io::Cursor;
use std::io::Read;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::{Error,Write};
use std::ops::Deref;
use serde_json::Value;
use std::collections::HashMap;
use request;
use std::thread;
use std::sync::{Mutex,Arc};
use byteorder::{BigEndian, ReadBytesExt};
use std::ops::DerefMut;
use response;
use std::str;
use crossbeam;

type handlerBox=Box<Fn(response::Response)+Send+Sync>;

pub struct Eventbus<'b>{
    client:TcpStream,
    handlers_access:Arc<Mutex<HashMap<String,Box<Fn(response::Response)+Send+'b+Sync>>>>
}

impl<'b> Eventbus<'b>{
    pub fn send_frame<'a>(&mut self, message:&'a str) ->Result<(),Error>{
        let p=super::protocol::Protocol{
            message_length:message.len() as i32,
            json_string:message.to_string()
        };
        let p_bytes=p.to_bytes();
        self.client.write(p_bytes.deref()).unwrap();
        Ok(())
    }

    pub fn connect(addr:&str)->Result<Eventbus<'static>,Error>{
        let stream=TcpStream::connect(addr).unwrap();
        let counter:Arc<Mutex<HashMap<String,Box<Fn(response::Response)+Send+'static+Sync>>>>= Arc::new(Mutex::new(HashMap::new()));
        let c_clone=counter.clone();
        let mut stream_clone=stream.try_clone().expect("clone failed");
        let read_thread=thread::spawn(move || {
            let mut length_bytes = vec![0u8; 4];
            loop {
                stream_clone.read_exact(&mut length_bytes).unwrap();
                let mut message_length=Cursor::new(length_bytes.clone());
                let l:i32=message_length.read_i32::<BigEndian>().unwrap();
                let l_usize=l as usize;
                if (l_usize as i32) < l {
                    panic!("packet is too big");
                }
                let mut message_bytes=vec![0u8;l_usize];
                stream_clone.read_exact(&mut message_bytes).unwrap();
                let s=str::from_utf8(&message_bytes).unwrap();
                let (res,add)=response::Response::from_str(s);
                {
                    let n=c_clone.lock().unwrap();
                    let v = &*n.get(&add).unwrap();
                    crossbeam::scope(|scope|{
                        scope.spawn(||{
                            v(res);
                        });
                    });
                }
            }
        });
        Ok(Eventbus{
            client:stream,
            handlers_access:counter
        })
    }

    pub fn send<T>(&mut self,address:&str,message:Value,handler:T)->Result<(),Error>
    where T:Fn(response::Response)+'static+Send+Sync
    {
        let req=request::Request::send(request::RegularRequestObject{
            address:address.to_string(),
            body:message,
            headers:None,
            replyAddress:Some(address.to_string())
        });
        {
            let mut p=self.handlers_access.lock().unwrap();
            (*p).insert(address.to_string(),Box::new(handler));
        }
        let s=req.to_json().to_string();
        self.send_frame(&s)
    }

    pub fn publish(&mut self,address:&str,message:Value)->Result<(),Error>{
        let req=request::Request::publish(request::RegularRequestObject{
            address:address.to_string(),
            body:message,
            headers:None,
            replyAddress:None
        });
        let s=req.to_json().to_string();
        self.send_frame(&s)
    }

    pub fn register(&mut self,address:&str,headers:Option<Value>)->Result<(),Error>{
        let req=request::Request::register(request::RegisterObject{
            address: address.to_string(),
            headers,
        });
        let s=req.to_json().to_string();
        self.send_frame(&s)
    }

    pub fn unregister(&mut self,address:&str,headers:Option<Value>)->Result<(),Error>{
        let req=request::Request::register(request::RegisterObject{
            address: address.to_string(),
            headers,
        });
        let s=req.to_json().to_string();
        self.send_frame(&s)
    }

    pub fn ping(&mut self)->Result<(),Error>{
        let s=request::Request::ping.to_json().to_string();
        self.send_frame(&s)
    }
}