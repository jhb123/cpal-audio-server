use std::mem::MaybeUninit;
use std::sync::Arc;
use std::{net::TcpStream, time::Instant};

use std::io::Write;

use audio_server::{create_audio_message, serialise_data, create_terminate_message};
use clap::Parser;
use cpal::{SupportedStreamConfig, Sample, FromSample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapRb, Consumer, SharedRb, Producer};

#[derive(Parser, Debug)]
#[command(version, about = "CPAL feedback example", long_about = None)]
struct Opt {
    /// The input audio device to use
    #[arg(short, long, value_name = "IN", default_value_t = String::from("default"))]
    input_device: String,

    /// The output audio device to use
    #[arg(short, long, value_name = "OUT", default_value_t = String::from("default"))]
    output_device: String,

    /// Specify the delay between input and output
    #[arg(short, long, value_name = "DELAY_MS", default_value_t = 1000.0)]
    latency: f32,


}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    


    let host = cpal::default_host();

    // Find devices.
    let input_device = if opt.input_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt.input_device).unwrap_or(false))
    }
    .expect("failed to find input device");


    println!("Using input device: \"{}\"", input_device.name()?);

    // We'll try and use the same configuration between streams to keep it simple.
    //let config: cpal::StreamConfig = input_device.default_input_config()?.into();
    let config = SupportedStreamConfig::new(
        1,
        cpal::SampleRate(44100),
         cpal::SupportedBufferSize::Range { min: 10_000, max: 20_000 }, 
         cpal::SampleFormat::F32
        );
    
    println!("{:?}",config);

    // Create a delay in case the input and output devices aren't synced. 
    // this can be used in the ring buff
    let latency_frames = (opt.latency / 1_000.0) * config.sample_rate().0 as f32;
    let latency_samples = latency_frames as usize * config.channels() as usize;

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    let start = Instant::now();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        //println!("Starting recording: {:?}", start.elapsed());
        producer.push_slice(data);
        //println!("Stopping recording: {:?}", start.elapsed());
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
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        opt.latency
    );
    input_stream.play()?;
    
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
    println!("record what you want to say");
    //let mut buf_writer = BufWriter::new();

    let dur = std::time::Duration::from_millis(4000);
//    std::thread::sleep();
    let start = Instant::now();

    let latency_dur = std::time::Duration::from_millis(opt.latency as u64);
    println!("Latency: {:?} ms",latency_dur);
    let mut buf = [0f32; 100000];
    //let mut buf: Vec<f32> = Vec::with_capacity(10000);
    let audio_latency = std::time::Duration::from_micros(100);

    while Instant::now() - start < dur {
        std::thread::sleep(audio_latency);

        let num_samples = consumer.pop_slice(&mut buf);
        println!("samples recorded {:?}", num_samples);
        if buf.len() != 0 {
            let msg = create_audio_message(&buf[0..num_samples]);
            let serialised = serialise_data(&msg);
            println!("serialised samples len {:?}", serialised.len());

            //buf_writer.write(&serialised);
            let res = stream.write(&serialised);
            match res {
                Ok(_) => (),
                Err(_) => println!("Oh no!"),
                 
            }
        }
    };

    let terminatation = create_terminate_message();
    let serialised = serialise_data(&terminatation);
    let _ = stream.write(&serialised);
    //buf_writer.write(&serialised);
    // Run for 3 seconds before closing.
    drop(input_stream);

    println!("Done!");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

// fn make_vec(ring_buffer: &mut Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>> ) -> Vec<f32>{
    
//     let mut output = Vec::<f32>::new();

//     while let Some(i) = ring_buffer.pop() {
//         //println!("popping data");
//         output.push(i);
//         //output.push(i);
//     }
//     //output.reverse();
//     output
   
// }