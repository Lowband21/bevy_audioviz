use bevy::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use std::sync::mpsc::Receiver;
use std::thread;

// Define a new Bevy event to communicate audio data to the main thread.
#[derive(Event)]
pub struct AudioProcessedEvent(pub Vec<Vec<f32>>);

#[derive(Debug)]
pub enum DeviceType {
    Input(),
    Output(),
}
// Adapted `stream_input` to use Bevy's event system.
// This function initializes the audio input stream and returns a receiver for audio events.
pub fn stream_input(
    device_type: DeviceType,
    buffer_size: usize, // Add this parameter to specify the buffer size
) -> Receiver<AudioProcessedEvent> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let host = cpal::default_host();
        let device = match device_type {
            DeviceType::Input() => host
                .default_input_device()
                .expect("No default input device"),
            DeviceType::Output() => host
                .default_output_device()
                .expect("No default output device"),
        };

        let config = device
            .default_input_config()
            .expect("Failed to get default input config");

        let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        // Create a buffer with the specified buffer size
                        let mut buffer = vec![0.0; buffer_size];

                        // Fill the buffer with the incoming audio data
                        for (i, &sample) in data.iter().enumerate().take(buffer_size) {
                            buffer[i] = sample;
                        }

                        // Send the buffer to the main Bevy thread through the channel
                        // Here the buffer is split into chunks of size that can be changed as needed
                        sender
                            .send(AudioProcessedEvent(
                                buffer.chunks_exact(4).map(|c| c.to_vec()).collect(),
                            ))
                            .expect("Failed to send audio data");
                    },
                    err_fn,
                    None,
                )
                .expect("Failed to build audio input stream"),
            _ => panic!("Unsupported sample format"),
        };

        stream.play().expect("Failed to play audio stream");
        thread::park();
    });

    receiver
}

// Define the error handling function for the audio stream.
fn err_fn(err: cpal::StreamError) {
    eprintln!("An error occurred on the audio stream: {}", err);
}

// Define a wrapper around `mpsc::Receiver` to make it a Bevy resource.
#[derive(Resource)]
pub struct AudioReceiver {
    pub receiver: Arc<Mutex<Receiver<AudioProcessedEvent>>>,
}

// Implement `FromWorld` for the new resource type.
impl FromWorld for AudioReceiver {
    fn from_world(_: &mut World) -> Self {
        // Here you would call your `stream_input` function to create the receiver.
        let (_sender, receiver) = channel();

        // Set up the audio streaming in a separate thread.
        // ...

        AudioReceiver {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}
// Bevy system to initialize audio capture and store the receiver in a resource.
pub fn audio_capture_startup_system(mut commands: Commands) {
    // Retrieve the receiver from the `stream_input` function.
    let audio_receiver = AudioReceiver {
        receiver: Arc::new(Mutex::new(stream_input(DeviceType::Output(), 2048))),
    };

    // Insert the receiver into Bevy's resource system for later access.
    commands.insert_resource(audio_receiver);
}
