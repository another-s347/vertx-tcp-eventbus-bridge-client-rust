#![allow(non_snake_case)]

use serde_json;
use serde_json::Value;

//Messages from the bridge -> client
#[derive(Debug, Clone)]
pub enum Response {
    ERR(ErrorType),
    MESSAGE(ResponseMessageObject),
    MessageFail(ResponseFailObject),
    PONG,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    AccessDenied,
    AddressRequired,
    UnknownAddress,
    UnknownType,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseMessageObject {
    pub address: String,
    pub body: Value,
    pub headers: Option<Value>,
    pub replyAddress: Option<String>,
    pub send: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseFailObject {
    pub failureCode: i32,
    pub failureType: String,
    pub message: String,
    pub sourceAddress: String,
}

impl Response {
    pub fn from_slice(s: &[u8]) -> (Response, String) {
        let v: Value = serde_json::from_slice(s).unwrap();
        let typ = v["type"].as_str().expect("type should be string");
        let addr = v["address"].as_str().expect("address should be string").to_string();
        match typ.as_ref() {
            "pong" => (Response::PONG, "".to_string()),
            "err" => {
                let err_msg = v["message"].as_str().expect("message should be string");
                match err_msg.as_ref() {
                    "access_denied" => (Response::ERR(ErrorType::AccessDenied), addr),
                    "address_required" => (Response::ERR(ErrorType::AddressRequired), addr),
                    "unknown_address" => (Response::ERR(ErrorType::UnknownAddress), addr),
                    "unknown_type" => (Response::ERR(ErrorType::UnknownType), addr),
                    _ => {
                        let j: ResponseFailObject = serde_json::from_slice(s).unwrap();
                        (Response::MessageFail(j), addr)
                    }
                }
            }
            "message" => {
                let j: ResponseMessageObject = serde_json::from_slice(s).unwrap();
                (Response::MESSAGE(j), addr)
            }
            _ => panic!(""),
        }
    }
}
