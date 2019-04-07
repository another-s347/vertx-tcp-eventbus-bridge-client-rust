use serde_json;
use serde_json::Value;

//Messages from the bridge -> client
#[derive(Debug,Clone)]
pub enum Response {
    ERR(ErrorType),
    MESSAGE(ResponseMessageObject),
    MESSAGE_FAIL(ResponseFailObject),
    PONG,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    access_denied,
    address_required,
    unknown_address,
    unknown_type,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseMessageObject {
    address: String,
    body: Value,
    headers: Option<Value>,
    replyAddress: Option<String>,
    send: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseFailObject {
    failureCode: i32,
    failureType: String,
    message: String,
    sourceAddress: String,
}

impl Response {
    pub fn from_slice(s: &[u8]) -> (Response, String) {
        let v: Value = serde_json::from_slice(s).unwrap();
        let typ = &v["type"].as_str().expect("type should be string");
        let addr= v["address"].as_str().expect("address should be string").to_string();
        match typ.as_ref() {
            "pong" => (Response::PONG,"".to_string()),
            "err" => {
                let err_msg = &v["message"].as_str().expect("message should be string");
                match err_msg.as_ref() {
                    "access_denied" => (Response::ERR(ErrorType::access_denied),addr),
                    "address_required" => (Response::ERR(ErrorType::address_required),addr),
                    "unknown_address" => (Response::ERR(ErrorType::unknown_address),addr),
                    "unknown_type" => (Response::ERR(ErrorType::unknown_type),addr),
                    _ => {
                        let j: ResponseFailObject = serde_json::from_slice(s).unwrap();
                        (Response::MESSAGE_FAIL(j),addr)
                    }
                }
            }
            "message" => {
                let j: ResponseMessageObject = serde_json::from_slice(s).unwrap();
                (Response::MESSAGE(j),addr)
            }
            _ => panic!(""),
        }
    }
}
