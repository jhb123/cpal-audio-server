#![feature(vec_into_raw_parts)]

pub mod audio {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/audio.items.rs"));
    }
}

use std::{mem::size_of, io::Cursor};
use audio::items;
use cpal::{SampleFormat, SupportedStreamConfig};
use prost::Message;

pub fn create_terminate_message()-> items::Data {
    let mut data: items::Data = items::Data::default();
    data.terminate_connection = true;
    data
}

pub fn create_config_message(cpal_config: &SupportedStreamConfig)-> items::Config{
    let mut cfg: items::Config = items::Config::default();

    cfg.encoding = encode_sample_format(&cpal_config.sample_format());
    cfg.channels = cpal_config.channels() as u32;
    cfg.sample_rate = cpal_config.sample_rate().0;

    cfg

}

pub fn create_audio_message<T>(input_data: &[T])-> items::Data where T:Clone {
    let mut data: items::Data = items::Data::default();

    // creates a message for sending over tcp. This takes in a slice of data
    // and returns a protocol buffer version of the data which can be serialised
    // with serialise_data.

    // the message has these parts: 
    // - a termination signal. This lets us tell the server when the session
    //   has ended
    // - the raw data. This is a byte array, and this was chosen to keep the 
    //   protocol buffer simple.

    // this message is not a termination message, so set this to false.
    data.terminate_connection = false;
    // this is is for setting the type of data.
    //data.message_type = std::any::type_name::<T>().to_string();

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

pub fn serialise<T: prost::Message>(data: &T) -> Vec<u8> {
    let mut buf= Vec::with_capacity(data.encoded_len());
    // this is safe since its reserved the size of the buffer
    data.encode(&mut buf).unwrap();
    buf
}

pub fn deserialise_config(buf: &[u8]) -> Result<items::Config, prost::DecodeError>{
    items::Config::decode(&mut Cursor::new(buf))
}

pub fn deserialise_data(buf: &[u8]) -> Result<items::Data, prost::DecodeError>{
    items::Data::decode(&mut Cursor::new(buf))
}

// pub fn serialise_data(data: &items::Data) -> Vec<u8> {
//     let mut buf= Vec::with_capacity(data.encoded_len());
//     // this is safe since its reserved the size of the buffer
//     data.encode(&mut buf).unwrap();
//     buf
// }

// pub fn serialise_config(data: &items::Config) -> Vec<u8> {
//     let mut buf= Vec::with_capacity(data.encoded_len());
//     // this is safe since its reserved the size of the buffer
//     data.encode(&mut buf).unwrap();
//     buf
// }



fn encode_sample_format(data_type: &SampleFormat)-> i32 {
    match data_type {
        SampleFormat::I8 => 0,
        SampleFormat::I16 => 1,
        SampleFormat::I32 => 2,
        SampleFormat::I64 => 3,
        SampleFormat::U8 => 4,
        SampleFormat::U16 => 5,
        SampleFormat::U32 => 6,
        SampleFormat::U64 => 7,
        SampleFormat::F32 => 8,
        SampleFormat::F64 => 9,
        _ => panic!("the data type {:?} is not supported",data_type),
    }
}

pub fn decode_sample_format(sample_format: i32) -> SampleFormat {
    match sample_format {
        0 => SampleFormat::I8,
        1 => SampleFormat::I16,
        2 => SampleFormat::I32,
        3 => SampleFormat::I64,
        4 => SampleFormat::U8,
        5 => SampleFormat::U16,
        6 => SampleFormat::U32,
        7 => SampleFormat::U64,
        8 => SampleFormat::F32,
        9 => SampleFormat::F64,
        _ => panic!("cannot decode this data type"),
    }

}

// pub trait as {
//     fn summarize(&self) -> String;
// }

// fn return_same_type<T>(value: &SampleFormat) -> T {
//     value
// }

// fn encode_sample_format(data_type: &SampleFormat)-> i32 {
//     match data_type {
//         SampleFormat::I8 => 0,
//         SampleFormat::I16 => 1,
//         SampleFormat::I32 => 2,
//         SampleFormat::I64 => 3,
//         SampleFormat::U8 => 4,
//         SampleFormat::U16 => 5,
//         SampleFormat::U32 => 6,
//         SampleFormat::U64 => 7,
//         SampleFormat::F32 => 8,
//         SampleFormat::F64 => 9,
//         _ => panic!("the data type {:?} is not supported",data_type),
//     }
// }