use std::collections::HashMap;
use std::io::Error as IoError;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};

use crossbeam::sync::ArcCell;
use futures::IntoFuture;
use futures::stream::Forward;
use futures::sync::mpsc::*;
use futures::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};
use serde_json::Value;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::io::{ErrorKind, ReadHalf, WriteHalf};
use tokio::io::AsyncRead;
use tokio::net::TcpStream;
use tokio::prelude::{Async, Stream};
use tokio::prelude::future::Future;

use crate::codec::{RequestCodec, ResponseCodec};
use crate::request;
use crate::request::Request;
use crate::response::Response;

pub struct Eventbus {
    tx: Arc<RwLock<HashMap<String, Sender>>>,
    writer: UnboundedSender<Request>,
}

pub struct EventbusWriteStream {
    inner: FramedWrite<WriteHalf<TcpStream>, RequestCodec>,
    rx: UnboundedReceiverWithError<Request>,
}

pub struct EventbusReadStream {
    reader: FramedRead<ReadHalf<TcpStream>, ResponseCodec>,
    tx: Arc<RwLock<HashMap<String, Sender>>>,
}

enum Sender {
    Unbounded(UnboundedSender<Response>),
    Oneshot(ArcCell<Option<OneshotSender<Response>>>),
}

impl Stream for EventbusReadStream {
    type Item = (Response, String);
    type Error = std::io::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.reader.poll() {
            Ok(Async::Ready(Some((response, address)))) => {
                self.send(&address, response.clone());
                return Ok(Async::Ready(Some((response, address))));
            }
            other => other
        }
    }
}

pub struct UnboundedReceiverWithError<T>(pub UnboundedReceiver<T>);

impl<T> Stream for UnboundedReceiverWithError<T> {
    type Item = T;
    type Error = IoError;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.0.poll() {
            Err(_) => {
                Err(IoError::new(ErrorKind::BrokenPipe, "UnboundedSender closed ?"))
            }
            Ok(other) => {
                Ok(other)
            }
        }
    }
}

impl IntoFuture for EventbusWriteStream {
    type Future = Forward<UnboundedReceiverWithError<Request>, FramedWrite<WriteHalf<TcpStream>, RequestCodec>>;
    type Item = (UnboundedReceiverWithError<Request>, FramedWrite<WriteHalf<TcpStream>, RequestCodec>);
    type Error = IoError;

    fn into_future(self) -> Self::Future {
        self.rx.forward(self.inner)
    }
}

pub struct ResponseStream {
    rx: UnboundedReceiver<Response>
}

pub struct ResponseFut {
    rx: OneshotReceiver<Response>
}

impl Future for ResponseFut {
    type Item = Response;
    type Error = IoError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.rx.poll() {
            Ok(i) => Ok(i),
            Err(_) => Err(IoError::new(ErrorKind::Other, "canceled"))
        }
    }
}

impl Stream for ResponseStream {
    type Item = Response;
    //todo: Error
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.rx.poll().map_err(|_| ())
    }
}

impl Eventbus {
    pub fn connect(address: IpAddr, port: u16) -> impl Future<Item=(Eventbus, EventbusReadStream, EventbusWriteStream), Error=IoError> {
        TcpStream::connect(&SocketAddr::new(address.clone(), port)).map(move |s| {
            let (write_tx, write_rx) = unbounded();
            let map = Arc::new(RwLock::new(HashMap::new()));
            let (r, w) = s.split();
            let reader = FramedRead::new(r, ResponseCodec);
            let writer = FramedWrite::new(w, RequestCodec);
            let read_stream = EventbusReadStream {
                reader,
                tx: map.clone(),
            };
            let write_stream = EventbusWriteStream {
                inner: writer,
                rx: UnboundedReceiverWithError(write_rx),
            };
            let eventbus = Eventbus {
                tx: map,
                writer: write_tx,
            };
            (eventbus, read_stream, write_stream)
        })
    }

    fn send_frame(&self, req: Request) -> Result<(), SendError<Request>> {
        self.writer.unbounded_send(req)
    }

    pub fn register(&self, address: String, headers: Option<Value>) -> Result<ResponseStream, SendError<Request>> {
        let req = request::Request::Register {
            address: address.to_string(),
            headers,
        };
        self.send_frame(req).map(|_| {
            let (tx, rx) = unbounded();
            let mut map = self.tx.write().unwrap();
            map.insert(address, Sender::Unbounded(tx));
            ResponseStream {
                rx
            }
        })
    }

    pub fn unregister(&self, address: String) {
        let req = request::Request::Unregister {
            address: address.to_string(),
            headers: None,
        };
        self.send_frame(req).unwrap();
        let mut map = self.tx.write().unwrap();
        map.remove(&address);
    }

    pub fn ping(&mut self) -> Result<(), SendError<Request>> {
        let s = request::Request::Ping;
        self.send_frame(s)
    }

    pub fn send(&self, address: String, message: Value) -> Result<(), SendError<Request>> {
        let req = request::Request::Send {
            address: address.to_string(),
            body: message,
            headers: None,
            replyAddress: Some(address.to_string()),
        };
        self.send_frame(req)
    }

    pub fn send_reply(&self, address: String, message: Value) -> Result<ResponseFut, SendError<Request>> {
        let req = request::Request::Send {
            address: address.to_string(),
            body: message,
            headers: None,
            replyAddress: Some(address.to_string()),
        };
        self.send_frame(req).map(|_| {
            let (tx, rx) = oneshot::channel();
            let mut map = self.tx.write().unwrap();
            map.insert(address, Sender::Oneshot(ArcCell::new(Arc::new(Some(tx)))));
            ResponseFut {
                rx
            }
        })
    }

    pub fn publish(&mut self, address: String, message: Value) -> Result<(), SendError<Request>> {
        let req = request::Request::Publish {
            address: address.to_string(),
            body: message,
            headers: None,
            replyAddress: None,
        };
        self.send_frame(req)
    }
}

impl EventbusReadStream {
    fn send(&mut self, address: &String, response: Response) {
        let remove = if let Some(tx) = self.tx.read().unwrap().get(address) {
            match tx {
                Sender::Unbounded(tx) => {
                    match tx.unbounded_send(response) {
                        Ok(_) => false,
                        Err(_) => true
                    }
                }
                Sender::Oneshot(cell) => {
//                    if let Some(tx) = cell.set(None) {
//                        tx.send(response);
//                    };
//                    true
                    let tx_opt = Arc::try_unwrap(cell.set(Arc::new(None))).unwrap();
                    if let Some(tx) = tx_opt {
                        tx.send(response).unwrap();
                    }
                    true
                }
            }
        } else { false };
        if remove {
            if let Ok(mut map_mut) = self.tx.write() {
                map_mut.remove(address);
            }
        }
    }
}