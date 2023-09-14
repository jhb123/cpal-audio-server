use std::{net::UdpSocket, time::Instant};
use std::io::Write;

use audio_server::audio::items::Config;
use audio_server::{create_audio_message, serialise, create_terminate_message, create_config_message};
use cpal::{SupportedStreamConfig, Device, SizedSample, StreamConfig};
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

    let config = input_device.default_input_config().unwrap();
    
    println!("Config: {:?}",config);

    let _ = match &config.sample_format() {
        cpal::SampleFormat::I8  => run::<i8>(config, &input_device),
        cpal::SampleFormat::I16 => run::<i16>(config, &input_device),
        cpal::SampleFormat::I32 => run::<i32>(config, &input_device),
        cpal::SampleFormat::I64 => run::<i64>(config, &input_device),
        cpal::SampleFormat::U8  => run::<u8>(config, &input_device),
        cpal::SampleFormat::U16 => run::<u16>(config, &input_device),
        cpal::SampleFormat::U32 => run::<u32>(config, &input_device),
        cpal::SampleFormat::U64 => run::<u64>(config, &input_device),
        cpal::SampleFormat::F32 => run::<f32>(config, &input_device),
        cpal::SampleFormat::F64 => run::<f64>(config, &input_device),
        _ => panic!("format not supported"),
    };

    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn run<T> (config: SupportedStreamConfig, input_device: &Device)-> anyhow::Result<()> 
where T: Default + Copy + SizedSample + Send + 'static
{
    let mut stream = UdpSocket::bind("127.0.0.1:43443").unwrap();
    stream.connect("127.0.0.1:43442");

    let msg = create_config_message(&config);
    let serialised = serialise(&msg);
    let res = stream.send(&serialised)?;

    let ring: HeapRb<T> = HeapRb::new(1024);
    let (mut producer, mut consumer) = ring.split();

    let input_data_fn = move |data: &[T], _: &cpal::InputCallbackInfo| {
        producer.push_slice(data);
    };


    let input_stream = input_device.build_input_stream(
        &config.into(),
        input_data_fn,
        err_fn,
        None)?;
    println!("Successfully built stream.");

    input_stream.play()?;
    
    println!("record what you want to say");
    //let mut buf_writer = BufWriter::new();

    let dur = std::time::Duration::from_millis(10000);
    let start = Instant::now();

    let mut buf: Vec<T> = vec![Default::default(); 10000];//Vec::<f32>::new();
    
    while Instant::now() - start < dur {

        let num_samples = consumer.pop_slice(&mut buf);
        if num_samples != 0 {
            let msg = create_audio_message(&buf[0..num_samples]);
            let serialised = serialise(&msg);
            let res = stream.send(&serialised);
            match res {
                Ok(_) => {},
                Err(_) => eprintln!("Issue writing to stream!"),
                 
            }
        }
    };

    let terminatation = create_terminate_message();
    let serialised = serialise(&terminatation);
    let _ = stream.send(&serialised);

    // this is cpals examples, but I don't know why. Isn't this dropped 
    // at the end of this function anyway?
    drop(input_stream);

    println!("Done!");
    Ok(())
}