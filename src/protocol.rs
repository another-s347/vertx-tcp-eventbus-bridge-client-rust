use bytes::{BytesMut,BufMut,Bytes};

pub struct Protocol{
    pub message_length:i32,
    pub json_string:String
}

impl Protocol{
    pub fn to_bytes(&self)->Bytes{
        let mut buf=BytesMut::with_capacity(4+self.message_length as usize);
        buf.put_i32_be(self.message_length);
        buf.put(self.json_string.clone());
        buf.freeze()
    }
}