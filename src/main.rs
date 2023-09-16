// to get reading byte buffers to a little endian to work with generic types
#![feature(generic_const_exprs)]

use audio_server::{deserialise_data, deserialise_config, decode_sample_format, audio::items::{Config, Endian}};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SupportedStreamConfig, SizedSample};
use eio::{ReadExt, FromBytes};
use ringbuf::{HeapRb, SharedRb, Producer};
use std::{
    net::UdpSocket, mem::{size_of, MaybeUninit}, sync::Arc,
};

fn main() {
    let listener = UdpSocket::bind("0.0.0.0:43442").unwrap();

    // there should be a nice way of shutting the server down ...
    loop {
        connection_handler(listener.try_clone().unwrap());
    }

}

fn connection_handler(stream: UdpSocket) -> anyhow::Result<()> {
    
    // first thing that happens is a configuration of the audio is recieved.
    // This is an very short byte message, but later on it might get more 
    // complex. For that reason, I've left quite a bit of redundancy...
    let mut buf = [0;100];
    let num_bytes = stream.recv(&mut buf).unwrap();
    let client_config = deserialise_config(&buf[0..num_bytes]).unwrap();

    println!("{:?}",client_config);

    // Set up the 
    // using a small buffer_size reduces the latency.

    let _ = match decode_sample_format(client_config.encoding) {
        cpal::SampleFormat::I8 => Ok(run::<i8>(client_config,stream)),
        cpal::SampleFormat::I16 => Ok(run::<i16>(client_config,stream)),
        cpal::SampleFormat::I32 => Ok(run::<i32>(client_config,stream)),
        cpal::SampleFormat::I64 => Ok(run::<i64>(client_config,stream)),
        cpal::SampleFormat::U8 => Ok(run::<u8>(client_config,stream)),
        cpal::SampleFormat::U16 => Ok(run::<u16>(client_config,stream)),
        cpal::SampleFormat::U32 => Ok(run::<u32>(client_config,stream)),
        cpal::SampleFormat::U64 => Ok(run::<u64>(client_config,stream)),
        cpal::SampleFormat::F32 => Ok(run::<f32>(client_config,stream)),
        cpal::SampleFormat::F64 => Ok(run::<f64>(client_config,stream)),
        _ => Err("Format not supported"),
    };
   
    println!("Finished");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn run<T>(client_config: Config, stream: UdpSocket)-> anyhow::Result<()> 
where T: SizedSample + Send + FromBytes< {size_of::<T>()} > + 'static
{

    //check what endian the client is using
    let client_endian = client_config.endian();
    //println!("{}",ce);
    //let client_endian_match_system = client_server_endian_match(&client_config);

    let config = SupportedStreamConfig::new(
        client_config.channels as u16,
        cpal::SampleRate(client_config.sample_rate),
        cpal::SupportedBufferSize::Range { min: 14, max: 512 }, 
        decode_sample_format(client_config.encoding)
        );

    // this is a pretty cool object. It provides Lock-free operations - they succeed or 
    // fail immediately without blocking or waiting. It is being written to from the 
    // UDP socket and read by CPAL when the system needs to read some more audio data.
    // And this is all done asynchronously! Setting the capacity to 1024 was semi-arbitary
    let ring: HeapRb<T> = HeapRb::new(1024);
    let (mut producer, mut consumer) = ring.split();

    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();
    println!("Using output device: \"{}\"", output_device.name()?);
    
    let channels = config.channels();
    let output_data_fn = move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
        for frame in data.chunks_mut(channels as usize){
            match consumer.pop() {
                Some(s) => {
                    for sample in frame.iter_mut() {
                        *sample = s;
                    }
                }
                None => {},
            }
        }
    };

    let output_stream = output_device.build_output_stream(
        &config.into(),
        output_data_fn,
        err_fn,
        None)?;

    output_stream.play()?;
        
    println!("listening for data");
    let mut buf: [u8; 10000] = [0; 10000];

    loop {
        // recieve some data. If there's an error, set size to 0.
        let size = stream.recv(&mut buf).unwrap_or_else(|err| {
            eprintln!("{}",err);
            0
        });

        if size > 0 {
            let deserialised = deserialise_data(&buf[0..size]);
            match deserialised {
                Ok(data) => {

                    let raw_data = data.message_data;
                    let _ = match client_endian {
                        Endian::Little => push_le_byte_data_to_ring_buffer(&mut producer, raw_data),
                        Endian::Big => push_be_byte_data_to_ring_buffer(&mut producer, raw_data),
                    };
                    //push_be_byte_data_to_ring_buffer(&mut producer, raw_data);
                    // let _ = match client_endian {
                    //     audio_server::Endian::BigEndian => push_be_byte_data_to_ring_buffer(&mut producer, raw_data),
                    //     audio_server::Endian::LittleEndian => push_le_byte_data_to_ring_buffer(&mut producer, raw_data),
                    // };

                    if data.terminate_connection {
                        break;
                    }
                }
                Err(msg) => eprintln!("{}",msg),
            }
        }
            
        
        
    }
    Ok(())
}

fn push_be_byte_data_to_ring_buffer<T>(
    producer: &mut Producer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>,
    byte_data: Vec<u8>) -> Result<(), std::io::Error> 
    where T: FromBytes< {size_of::<T>()} >
{
    // this function takes in a vector of bytes and decodes them to the type T
    // in little endian order. This data is pushed to a ring buffer for 
    // asychronos collection elsewhere.

    for chunk in byte_data.chunks_exact(size_of::<T>()){
        let sample = std::io::Cursor::new(chunk).read_be()?;
        producer.push(sample).unwrap_or_else(
            |_| {eprintln!("Failed to push sample")});
    }
    Ok(())
}

fn push_le_byte_data_to_ring_buffer<T>(
    producer: &mut Producer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>,
    byte_data: Vec<u8>) -> Result<(), std::io::Error> 
    where T: FromBytes< {size_of::<T>()} >
{
    // this function takes in a vector of bytes and decodes them to the type T
    // in little endian order. This data is pushed to a ring buffer for 
    // asychronos collection elsewhere.

    for chunk in byte_data.chunks_exact(size_of::<T>()){
        let sample = std::io::Cursor::new(chunk).read_le()?;
        producer.push(sample).unwrap_or_else(
            |_| {/* put something here to handle errors */ });
    }
    Ok(())
}
