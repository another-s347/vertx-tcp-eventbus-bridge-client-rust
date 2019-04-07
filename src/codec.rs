use std::io::Error as IoError;
use std::io::ErrorKind;
use std::u32;

use bytes::BigEndian;
use bytes::BufMut;
use bytes::ByteOrder;
use bytes::BytesMut;
use tokio::codec::{Decoder, Encoder};

use crate::request::Request;
use crate::response::Response;

pub struct RequestCodec;

pub struct ResponseCodec;

impl Encoder for RequestCodec {
    type Item = Request;
    type Error = IoError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = item.to_json().to_string();
        let len = data.len();
        if len > u32::max_value() as usize {
            // to long
            return Err(IoError::new(ErrorKind::InvalidInput, "len is too long (> u32::MAX)"));
        }
        dst.reserve(4 + len);
        dst.put_u32_be(len as u32);
        dst.put(data);
        Ok(())
    }
}

impl Decoder for ResponseCodec {
    type Item = (Response, String);
    type Error = IoError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = BigEndian::read_u32(src.as_ref()) as usize;
        if len + 4 > src.len() {
            return Ok(None);
        }
        let mut rest = src.split_to(4 + len);
        let rest = rest.split_off(4);
        let result = Response::from_slice(rest.as_ref());
        Ok(Some(result))
    }
}