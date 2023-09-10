#![feature(vec_into_raw_parts)]

pub mod audio {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/audio.items.rs"));
    }
}

use std::io::Cursor;

use audio::items;
use prost::Message;

pub fn create_message<T>(input_data: &[T])-> items::Data where T:Clone {
    let mut data = items::Data::default();

    
    data.message_type = std::any::type_name::<T>().to_string();

    let (ptr, len, cap) = input_data.to_vec().into_raw_parts();

    let rebuilt = unsafe {
        // We can now make changes to the components, such as
        // transmuting the raw pointer to a compatible type.
        let ptr = ptr as *mut u8;
    
        Vec::from_raw_parts(ptr, len, cap)
    };
    data.message_data = rebuilt;

    data
}

pub fn serialise_data(data: &items::Data) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(data.encoded_len());
    // this is safe since its reserved the size of the buffer
    data.encode(&mut buf).unwrap();
    buf
    //let arr: &[u8] = &buf;
    //arr
}

pub fn deserialise_data(buf: &[u8]) -> Result<items::Data, prost::DecodeError>{
    items::Data::decode(&mut Cursor::new(buf))
}

// fn convert_to_bytes<T>(var: T)-> &[u8]{
//     &var.to_le_bytes()
// }
