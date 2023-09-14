use anyhow::Error;
use audio_server::{deserialise_data, deserialise_config, decode_sample_format};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SupportedStreamConfig, Sample, FromSample, Device, SizedSample};
use ringbuf::HeapRb;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:43442").unwrap();

    for stream in listener.incoming() {
        println!("Connection established!");

        let stream = stream.unwrap();
        stream.set_nodelay(true);
        let _ = connection_handler(stream);
    }
}

fn connection_handler(mut stream: TcpStream) -> anyhow::Result<()> {
    //let ring = StaticRb::<f32,1024>::default();

    // first thing that happens is a configuration of the audio is recieved.
    let mut buf = [0;1000];
    let num_bytes = stream.read(&mut buf).unwrap();
    let client_config = deserialise_config(&buf[0..num_bytes]).unwrap();

    println!("{:?}",client_config);

    let config = SupportedStreamConfig::new(
        client_config.channels as u16,
        cpal::SampleRate(client_config.sample_rate),
        cpal::SupportedBufferSize::Range { min: 14, max: 128 }, 
        decode_sample_format(client_config.encoding)
        );
    
    println!();

    let res = match config.sample_format() {
        cpal::SampleFormat::I8 => Ok(run::<i8>(config,stream)),
        cpal::SampleFormat::I16 => Ok(run::<i16>(config,stream)),
        cpal::SampleFormat::I32 => Ok(run::<i32>(config,stream)),
        cpal::SampleFormat::I64 => Ok(run::<i64>(config,stream)),
        cpal::SampleFormat::U8 => Ok(run::<u8>(config,stream)),
        cpal::SampleFormat::U16 => Ok(run::<u16>(config,stream)),
        cpal::SampleFormat::U32 => Ok(run::<u32>(config,stream)),
        cpal::SampleFormat::U64 => Ok(run::<u64>(config,stream)),
        cpal::SampleFormat::F32 => Ok(run::<f32>(config,stream)),
        cpal::SampleFormat::F64 => Ok(run::<f64>(config,stream)),
        _ => Err("Format not supported"),
    };
   
    if res.is_err() {println!("Format not support")}

    println!("Finished");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn run<T>(config: SupportedStreamConfig, mut stream: TcpStream)-> anyhow::Result<()> 
where T: Default + Copy + SizedSample + Send + 'static
{

    let channels = config.channels();
    let ring: HeapRb<T> = HeapRb::new(1024);
    let (mut producer, mut consumer) = ring.split();

    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();
    println!("Using output device: \"{}\"", output_device.name()?);
    

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
        // let mut line = String::new();
        let res = stream.read(&mut buf);
        match res {
            Ok(size) => {
                let deserialised = deserialise_data(&buf[0..size]);
                match deserialised {
                    Ok(data) => {

                        let raw_data = data.message_data;
                        unsafe {
                            let (_prefix, decoded_buf, _suffix) = raw_data.align_to::<T>();
                            for &sample in decoded_buf.iter() {
                                producer.push(sample);
                            }
                        }
                        if data.terminate_connection == true {
                            break;
                        }
                    }
                    Err(_) => println!("decode error to deserialise"),
                }
            }
            Err(_) => {
                println!("no data")
            }
        }
    }
    Ok(())
}