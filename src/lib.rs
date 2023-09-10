#![feature(vec_into_raw_parts)]

pub mod audio {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/audio.items.rs"));
    }
}

use std::{mem::size_of, io::Cursor};
use audio::items;
use prost::Message;

pub fn create_terminate_message()-> items::Data {
    let mut data: items::Data = items::Data::default();
    data.terminate_connection = true;
    data
}

pub fn create_audio_message<T>(input_data: &[T])-> items::Data where T:Clone {
    let mut data: items::Data = items::Data::default();

    // creates a message for sending over tcp. This takes in a slice of data
    // and returns a protocol buffer version of the data which can be serialised
    // with serialise_data.

    // the message has three parts: 
    // - a termination signal. This lets us tell the server when the session
    //   has ended
    // - a type (that is represented by a string, but I'll change this soon!).
    // - the raw data. This is a byte array, and this was chosen to keep the 
    //   protocol buffer simple.

    // this message is not a termination message, so set this to false.
    data.terminate_connection = false;
    // this is is for setting the type of data.
    data.message_type = std::any::type_name::<T>().to_string();

    let (ptr, len, cap) = input_data.to_vec().into_raw_parts();

    let rebuilt = unsafe {
        // transmute the raw pointer to a compatible type.
        let ptr = ptr as *mut u8;

        // make sure the whole input_data is copied. the size of u8 is 1, so the
        // scaling factor for the byte vector's len is the input data's length
        // multiplied by the size of input vectors the type.
        let input_type_size = size_of::<T>();
    
        Vec::from_raw_parts(ptr, len*input_type_size, cap)
    };
    data.message_data = rebuilt;
    data
}

pub fn serialise_data(data: &items::Data) -> Vec<u8> {
    // this is copied from the docs:
    // https://github.com/danburkert/snazzy/blob/master/src/lib.rs
    let mut buf = Vec::new();
    buf.reserve(data.encoded_len());
    // this is safe since its reserved the size of the buffer
    data.encode(&mut buf).unwrap();
    buf
}

pub fn deserialise_data(buf: &[u8]) -> Result<items::Data, prost::DecodeError>{
    items::Data::decode(&mut Cursor::new(buf))
}

// fn convert_to_bytes<T>(var: T)-> &[u8]{
//     &var.to_le_bytes()
// }
