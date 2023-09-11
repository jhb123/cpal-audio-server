# Overview

This is an experimental program for streaming audio. Currently, it's a literal echo server. It runs on `127.0.0.1:8000`.

Usage:
 - start the server with `cargo run --bin server`
 - start the demo client with `cargo run --bin client`
 - send a message and hear echo'd response!

The client will run for 10s. Make sure you have headphones in!

# Latency measurement
## Result
I've measured 70 milliseconds of delay between 
 - The sound being picked up by my Mac's microphone.
 - The audio data being collected by the client.
 - The client serialising the data and sending it over a TCP connection to my local network.
 - The server receiving the data from the local network.
 - The data being deserialised.
 - The data being sent to my Mac's audio output.
   
This is surprisingly high. A rough calculation of the latency involves:
 - the delay between microhones picking up the same sound. There is roughly 3ms of delay per metre which the sound travels, so this is negligible.
 - the buffered samples. A sample rate of 44100 samples/second and a buffer size of e.g. 128 samples gives 3ms. This is unlikely to be the cause.
 - the delay in cables and stray capacitance. I haven't calculated this but milliseconds is quite a lot.
 - The rust programs. Putting in some printlns makes me think this only adds up to a few milliseconds.


## Method
This measurement requires:
- A metronome. A loud, fairly high pitched click is good.
- A microphone. I used a passive microphone.
- An oscilloscope. The experiment done in this readme describes how to use the Hameg 203-4.

Ensure the time base is zero'd. Connect the microphone to the CH1 of the oscilloscope and the audio output of the computer running the server to Ch2. Set the oscilloscope to trigger from Ch1. Make sure the trigger is set to normal mode (not automatic mode) and uses the positive slope. Place the microphone near the metronome. Adjust the the level on the trigger until you see a stable signal appearing on the scope. Ch1 was set to DC coupling (to avoid capacitance issues) and 10mV/cm. Ch2 was set to 50mV/cm and DC coupled. The oscilloscope was used in dual+chop mode to display both signals at the same time. The time base was set to 10ms/cm. To measure the delay, read the x position of the signal measured on Ch2.

This is the bare essentials, and you'd get a much nicer measurement by connecting the signals to the oscilloscope via a transimpedance amplifier and having a more "pure" sounding metronome which can go to higher frequencies.
