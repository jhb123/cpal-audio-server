use std::{net::TcpStream, time::Instant};
use std::io::Write;

use audio_server::{create_audio_message, serialise_data, create_terminate_message};
use cpal::SupportedStreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;

fn main() -> anyhow::Result<()> {
    //let opt = Opt::parse();

    let host = cpal::default_host();

    // Find devices.
    let input_device =  host
        .default_input_device()
        .expect("failed to find input device");


    println!("Using input device: \"{}\"", input_device.name()?);

    // We'll try and use the same configuration between streams to keep it simple.
    //let config: cpal::StreamConfig = input_device.default_input_config()?.into();
    let config = SupportedStreamConfig::new(
        1,
        cpal::SampleRate(44100),
         cpal::SupportedBufferSize::Range { min: 16, max: 128 }, 
         cpal::SampleFormat::F32
        );
    
    println!("{:?}",config);

    
    // The buffer to share samples
    let ring: HeapRb<f32> = HeapRb::new(1024);
    let (mut producer, mut consumer) = ring.split();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        producer.push_slice(data);
    };


    println!(
        "Attempting to build streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(
        &config.into(),
        input_data_fn,
        err_fn,
        None)?;
    println!("Successfully built stream.");

    input_stream.play()?;
    
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();

    println!("record what you want to say");
    //let mut buf_writer = BufWriter::new();

    let dur = std::time::Duration::from_millis(10000);
    let start = Instant::now();

    let mut buf = [0f32; 10000];

    while Instant::now() - start < dur {

        let num_samples = consumer.pop_slice(&mut buf);
        if num_samples != 0 {
            let msg = create_audio_message(&buf[0..num_samples]);
            let serialised = serialise_data(&msg);
            let res = stream.write(&serialised);
            match res {
                Ok(_) => (),
                Err(_) => eprintln!("Oh no!"),
                 
            }
        }
    };

    let terminatation = create_terminate_message();
    let serialised = serialise_data(&terminatation);
    let _ = stream.write(&serialised);
    drop(input_stream);

    println!("Done!");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}