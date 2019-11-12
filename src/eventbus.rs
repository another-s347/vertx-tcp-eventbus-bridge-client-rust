use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use serde_json::Value;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::net::TcpStream;
use futures::select;
use crate::codec::{RequestCodec, ResponseCodec};
use crate::request;
use crate::request::Request;
use crate::response::Response;
use futures::StreamExt;
use futures::SinkExt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::oneshot;

/// The core struct to communicate with vert.x eventbus.
/// Can be created by calling `Eventbus::connect`.
pub struct Eventbus {
    writer: tokio::sync::mpsc::Sender<_M>,
}

pub struct Driver {
    channel: tokio::sync::mpsc::Receiver<_M>,
    tcpstream:TcpStream
}

impl Driver {
    pub async fn run(mut self) {
        let mut map = HashMap::new();
        let (r,w) = self.tcpstream.split();
        let reader = FramedRead::new(r, ResponseCodec);
        let mut writer = FramedWrite::new(w, RequestCodec);
        let mut reader = reader.fuse();
        let mut channel = self.channel.fuse();
        select! {
            response = reader.next() => {
                if let Some(Ok((response,address))) = response {
                    match map.entry(address) {
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            if match entry.get_mut() {
                                Sender::Channel(channel) => {
                                    channel.send(response).await.unwrap();
                                    false
                                }
                                Sender::Oneshot(oneshot) => {
                                    oneshot.take().unwrap().send(response).unwrap();
                                    true
                                }
                            } {
                                entry.remove();
                            }
                        }
                        _ => {}
                    }
                }
            },
            msg = channel.next() => {
                match msg {
                    Some(_M::Request(request)) => { writer.send(request).await.unwrap(); },
                    Some(_M::ReplyOneshot(address,oneshot)) => {
                        map.insert(address,Sender::Oneshot(Some(oneshot)));
                    },
                    Some(_M::ReplyStream(address,receiver)) => {
                        map.insert(address,Sender::Channel(receiver));
                    }
                    Some(_M::Remove(address)) => {
                        map.remove(&address);
                    }
                    None => {}
                }
            }
            default => {

            }
        }
    }
}

enum _M {
    Request(Request),
    ReplyOneshot(String,tokio::sync::oneshot::Sender<Response>),
    ReplyStream(String,tokio::sync::mpsc::Sender<Response>),
    Remove(String)
}

enum Sender {
    Channel(tokio::sync::mpsc::Sender<Response>),
    Oneshot(Option<tokio::sync::oneshot::Sender<Response>>),
}

impl Eventbus {
    pub async fn connect(address: IpAddr, port: u16) -> std::io::Result<(Eventbus,Driver)> {
        let tcpstream = TcpStream::connect(&SocketAddr::new(address.clone(), port)).await?;
        let (write_tx, write_rx) = tokio::sync::mpsc::channel(1024);
        let eventbus = Eventbus {
            writer: write_tx,
        };
        let driver = Driver {
            channel: write_rx,
            tcpstream
        };
        Ok((eventbus, driver))
    }

    async fn send_frame(&mut self, req: Request) -> Result<(), SendError> {
        self.writer.send(_M::Request(req)).await
    }

    pub async fn register(&mut self, address: String, headers: Option<Value>) -> Result<Receiver<Response>, SendError> {
        let req = request::Request::Register {
            address: address.to_string(),
            headers,
        };
        self.send_frame(req).await?;
        let (tx, rx) = channel(1024);
        self.writer.send(_M::ReplyStream(address.to_string(),tx)).await?;
        Ok(rx)
    }

    pub async fn unregister(&mut self, address: String) -> Result<(), SendError> {
        let req = request::Request::Unregister {
            address: address.to_string(),
            headers: None,
        };
        self.send_frame(req).await?;
        self.writer.send(_M::Remove(address)).await
    }

    pub async fn ping(&mut self) -> Result<(), SendError> {
        let s = request::Request::Ping;
        self.send_frame(s).await
    }

    /// send with no reply
    pub async fn send(&mut self, address: String, message: Value) -> Result<(), SendError> {
        let req = request::Request::Send {
            address: address.to_string(),
            body: message,
            headers: None,
            replyAddress: Some(address.to_string()),
        };
        self.send_frame(req).await
    }

    /// send with reply
    pub async fn send_reply(&mut self, address: String, message: Value) -> Result<tokio::sync::oneshot::Receiver<Response>, SendError> {
        let req = request::Request::Send {
            address: address.clone(),
            body: message,
            headers: None,
            replyAddress: Some(address.clone()),
        };
        self.send_frame(req).await?;
        let (tx, rx) = oneshot::channel();
        self.writer.send(_M::ReplyOneshot(address,tx)).await?;
        Ok(rx)
    }

    pub async fn publish(&mut self, address: String, message: Value) -> Result<(), SendError> {
        let req = request::Request::Publish {
            address: address.to_string(),
            body: message,
            headers: None,
            replyAddress: None,
        };
        self.send_frame(req).await
    }
}