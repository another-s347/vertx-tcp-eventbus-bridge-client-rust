use serde_json::Value;
use std::collections::HashMap;

//Message from the client -> bridge
pub enum Request{
    send(RegularRequestObject),
    publish(RegularRequestObject),
    register(RegisterObject),
    unregister(RegisterObject),
    ping
}

pub struct RegularRequestObject{
    pub address:String,
    pub body:Value,
    pub headers:Option<Value>,
    pub replyAddress:Option<String>
}

pub struct RegisterObject{
    pub address:String,
    pub headers:Option<Value>
}

impl Request{
    pub fn to_json(&self)->Value{
        match self {
            Request::send(ref obj)=>{
                let mut v:HashMap<String,Value>=HashMap::new();
                v.insert("type".to_owned(),json!("send"));
                v.insert("address".to_owned(),json!(obj.address));
                v.insert("body".to_owned(),obj.body.clone());
                if obj.headers.is_some() {
                    v.insert("headers".to_owned(),obj.headers.clone().unwrap());
                }
                if obj.replyAddress.is_some() {
                    v.insert("replyAddress".to_owned(),json!(obj.replyAddress.clone().unwrap()));
                }
                json!(v)
            }
            Request::publish(ref obj)=>{
                let mut v:HashMap<String,Value>=HashMap::new();
                v.insert("type".to_owned(),json!("publish"));
                v.insert("address".to_owned(),json!(obj.address));
                v.insert("body".to_owned(),obj.body.clone());
                if obj.headers.is_some() {
                    v.insert("headers".to_owned(),obj.headers.clone().unwrap());
                }
                if obj.replyAddress.is_some() {
                    v.insert("replyAddress".to_owned(),json!(obj.replyAddress.clone().unwrap()));
                }
                json!(v)
            }
            Request::register(ref obj)=>{
                let mut v:HashMap<String,Value>=HashMap::new();
                v.insert("type".to_owned(),json!("register"));
                v.insert("address".to_owned(),json!(obj.address));
                if obj.headers.is_some() {
                    v.insert("headers".to_owned(),obj.headers.clone().unwrap());
                }
                json!(v)
            }
            Request::unregister(ref obj)=>{
                let mut v:HashMap<String,Value>=HashMap::new();
                v.insert("type".to_owned(),json!("unregister"));
                v.insert("address".to_owned(),json!(obj.address));
                if obj.headers.is_some() {
                    v.insert("headers".to_owned(),obj.headers.clone().unwrap());
                }
                json!(v)
            }
            &Request::ping=>{
                json!({
                    "type":"ping"
                })
            }
        }
    }
}