#![allow(non_snake_case)]

use std::collections::HashMap;
use serde_json::Value;

//Message from the client -> bridge
pub enum Request {
    Send {
        address: String,
        body: Value,
        headers: Option<Value>,
        replyAddress: Option<String>,
    },
    Publish {
        address: String,
        body: Value,
        headers: Option<Value>,
        replyAddress: Option<String>,
    },
    Register {
        address: String,
        headers: Option<Value>,
    },
    Unregister {
        address: String,
        headers: Option<Value>,
    },
    Ping,
}

impl Request {
    pub fn to_json(&self) -> Value {
        match self {
            Request::Send {
                address,
                body,
                headers,
                replyAddress
            } => {
                let mut v: HashMap<String, Value> = HashMap::new();
                v.insert("type".to_owned(), json!("send"));
                v.insert("address".to_owned(), json!(address));
                v.insert("body".to_owned(), body.clone());
                if headers.is_some() {
                    v.insert("headers".to_owned(), headers.clone().unwrap());
                }
                if replyAddress.is_some() {
                    v.insert("replyAddress".to_owned(), json!(replyAddress.clone().unwrap()));
                }
                json!(v)
            }
            Request::Publish {
                address,
                body,
                headers,
                replyAddress
            } => {
                let mut v: HashMap<String, Value> = HashMap::new();
                v.insert("type".to_owned(), json!("publish"));
                v.insert("address".to_owned(), json!(address));
                v.insert("body".to_owned(), body.clone());
                if headers.is_some() {
                    v.insert("headers".to_owned(), headers.clone().unwrap());
                }
                if replyAddress.is_some() {
                    v.insert("replyAddress".to_owned(), json!(replyAddress.clone().unwrap()));
                }
                json!(v)
            }
            Request::Register {
                address,
                headers
            } => {
                let mut v: HashMap<String, Value> = HashMap::new();
                v.insert("type".to_owned(), json!("register"));
                v.insert("address".to_owned(), json!(address));
                if headers.is_some() {
                    v.insert("headers".to_owned(), headers.clone().unwrap());
                }
                json!(v)
            }
            Request::Unregister {
                address,
                headers
            } => {
                let mut v: HashMap<String, Value> = HashMap::new();
                v.insert("type".to_owned(), json!("unregister"));
                v.insert("address".to_owned(), json!(address));
                if headers.is_some() {
                    v.insert("headers".to_owned(), headers.clone().unwrap());
                }
                json!(v)
            }
            &Request::Ping => {
                json!({
                    "type":"ping"
                })
            }
        }
    }
}