use audio_server::deserialise_data;
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SupportedStreamConfig, Sample, FromSample};
use ringbuf::HeapRb;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
        println!("Connection established!");

        let stream = stream.unwrap();
        let _ = connection_handler(stream);
    }
}

fn connection_handler(mut stream: TcpStream) -> anyhow::Result<()> {
    //let ring = StaticRb::<f32,1024>::default();
    let ring: HeapRb<f32> = HeapRb::new(1024);
    let (mut producer, mut consumer) = ring.split();

    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();
    println!("Using output device: \"{}\"", output_device.name()?);
    // let config = output_device.default_output_config().unwrap();
    let config = SupportedStreamConfig::new(
        1,
        cpal::SampleRate(44100),
        cpal::SupportedBufferSize::Range { min: 14, max: 128 }, 
        cpal::SampleFormat::F32
        );
    println!("{:?}",config);

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        
        for frame in data.chunks_mut(1){
            match consumer.pop() {
                Some(sample) => {
                    let value: f32 = f32::from_sample(sample);
                    for sample in frame.iter_mut() {
                        *sample = value;
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

        
    //let mut buf_reader = BufReader::new(stream);
    println!("listening for data");
    let mut buf: [u8; 10000] = [0; 10000];

    let mut full_recording = Vec::<f32>::new();

    loop {
        // let mut line = String::new();
        let res = stream.read(&mut buf);
        match res {
            Ok(size) => {
                let deserialised = deserialise_data(&buf[0..size]);
                match deserialised {
                    Ok(data) => {

                        let _data_type = data.message_type;
                        let raw_data = data.message_data;
                        unsafe {
                            let (_prefix, decoded_buf, _suffix) = raw_data.align_to::<f32>();
                            for &sample in decoded_buf.iter() {
                                producer.push(sample);
                                //full_recording.push(sample)

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
    println!("finished recording");
    println!("samples: {}",full_recording.len());
    println!("Finished");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}


fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> Option<f32>)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        match next_sample() {
            Some(s) => {
                let value: T = T::from_sample(s);
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            },
            None => {eprintln!()},
        }
        
    }
}