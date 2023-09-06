use std::{net::{TcpStream, TcpListener}, io::{BufReader, BufRead, BufWriter, Read}};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};
use ringbuf::HeapRb;

fn main(){

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
        println!("Connection established!");

        let stream = stream.unwrap();
        connection_handler(stream);
    }

}


fn connection_handler(mut stream : TcpStream)-> anyhow::Result<()> {
    let ring = HeapRb::<f32>::new(512 * 2);
    let (mut producer, mut consumer) = ring.split();

    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();
    println!("Using output device: \"{}\"", output_device.name()?);
    let config: cpal::StreamConfig = output_device.default_output_config()?.into();
    let mut buf_raw = &mut [0; 128];
    //let mut buf: &mut [f32; 512] = &mut [0.0; 128*4];
    println!("Got here");
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data.iter_mut() {
            *sample = match consumer.pop() {
                Some(s) => {
                    println!("Outputting data");
                    s
                },
                None => {
                    0.0
                }
            };
        }
    };
    let output_stream = output_device.build_output_stream(
        &config, 
        output_data_fn, 
        err_fn,
        None)?;

    println!("creating output stream for data");
    output_stream.play()?;
    //run::<i32>(&device, &config.into());
    println!("listening for data");
    unsafe {
        loop {
            // let mut line = String::new();
            //let len = buf_reader.read().unwrap();
            
            stream.read(buf_raw);
            println!("Got data");
            let (prefix, buf, suffix) = buf_raw.align_to::<f32>(); 
            for &sample in buf{
                producer.push(sample);
            }   
        }
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
