use crate::cfg::MyConfig;
use crate::CfgResource;
use bevy::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::thread::JoinHandle;

use crate::visualization::VisualizationType;

#[derive(Event, Debug)]
pub struct AudioProcessedEvent {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

#[derive(Debug)]
pub enum DeviceType {
    Input,
    Output,
}

// Define a wrapper around `mpsc::Receiver` to make it a Bevy resource.
#[derive(Resource)]
pub struct AudioReceiver {
    pub receiver: Arc<Mutex<Receiver<AudioProcessedEvent>>>,
    pub thread_handle: Option<JoinHandle<()>>,
}

impl FromWorld for AudioReceiver {
    fn from_world(world: &mut World) -> Self {
        // Retrieve or insert a new run flag into the world
        let run_flag = world
            .get_resource_or_insert_with(|| AudioThreadFlag(Arc::new(AtomicBool::new(true))))
            .0
            .clone();

        let config = world.get_resource::<CfgResource>().unwrap().0.clone();

        let (audio_receiver, thread_handle) = if config.mic_mode {
            // Pass the run flag to the stream_input function
            stream_input(DeviceType::Input, run_flag, &config)
        } else {
            // Pass the run flag to the stream_input function
            stream_input(DeviceType::Output, run_flag, &config)
        };

        AudioReceiver {
            receiver: Arc::new(Mutex::new(audio_receiver)),
            thread_handle: Some(thread_handle),
        }
    }
}

// Define a simple wrapper around Arc<AtomicBool> to make it a Bevy resource.
#[derive(Resource)]
pub struct AudioThreadFlag(pub Arc<AtomicBool>);
pub fn stream_input(
    device_type: DeviceType,
    run_flag: Arc<AtomicBool>, // Accept the run flag as a parameter
    config: &MyConfig,
) -> (Receiver<AudioProcessedEvent>, JoinHandle<()>) {
    let (sender, receiver) = channel();
    let rf_closure = run_flag.clone(); // Clone for the closure
    let config = config.clone();

    let thread_handle = thread::spawn(move || {
        //for host in cpal::available_hosts() {
        //    println!("{:#?}", host);
        //}
        let host = cpal::default_host();

        let devices = match device_type {
            DeviceType::Input => host.input_devices().expect("No default input device"),
            DeviceType::Output => host.output_devices().expect("No default output device"),
        };

        let mut device = host.default_output_device().unwrap();
        println!("Placeholder");

        if let Some(configured_device) = config.device.clone() {
            for (device_index, dev) in devices.enumerate() {
                //println!("Device {}: {}", device_index, dev.name().unwrap());
                if dev.name().unwrap() == configured_device {
                    device = dev;
                    println!("Selected Device: {}", device.name().unwrap());
                }
            }
        }
        //println!("Spawned thread");

        let config = match device_type {
            DeviceType::Input => device
                .default_input_config()
                .expect("Failed to get default input config"),
            DeviceType::Output => device
                .default_output_config()
                .expect("Failed to get default output config"),
        };

        let config_clone = config.clone();
        let supported_buffer_size: cpal::SupportedBufferSize =
            config_clone.buffer_size().to_owned();
        let channels = config_clone.channels();
        println!("Config used has {} channels", channels);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if !rf_closure.load(Ordering::SeqCst) {
                        // If the run flag is false, return early.
                        return;
                    }
                    //println!("Building audio event");

                    match supported_buffer_size {
                        cpal::SupportedBufferSize::Range { min, max } => {
                            if data.len() > max as usize && data.len() < min as usize {
                                eprintln!("Buffer ({}) is outside of range: {}, {}", data.len(), min, max);
                                return;
                            }
                            let buffer: Vec<f32> = data.iter().cloned().collect();
                            // Initialize vectors for left and right channels
                            let mut left_channel = Vec::with_capacity(data.len() / 2);
                            let mut right_channel = Vec::with_capacity(data.len() / 2);

                            // Deinterlace the buffer into separate channels
                            for chunk in data.chunks_exact(2) {
                                if let [left_sample, right_sample] = *chunk {
                                    left_channel.push(left_sample);
                                    // Set the right channel to zero for testing
                                    right_channel.push(right_sample);
                                }
                            }
                            //println!("Buffer {:#?}", audio_event);
                            // Create the audio event with the deinterlaced channel data
                            let audio_event = AudioProcessedEvent {
                                left: left_channel,
                                right: right_channel,
                            };


                            if sender.send(audio_event).is_err() {
                                eprintln!("The receiver has been dropped, terminating audio input stream.");
                                rf_closure.store(false, Ordering::SeqCst); // Signal the thread to exit
                                return; // Exit early to avoid further processing
                            }
                        }
                        cpal::SupportedBufferSize::Unknown => {
                            panic!("Buffer size is unknown");

                        }
                    }

                },
                err_fn,
                None,
            )
            .expect("Failed to build audio input stream");

        stream.play().expect("Failed to play audio stream");

        // Loop until the run flag is set to false.
        while run_flag.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(10));
        }

        // Perform any necessary cleanup here, if required
        drop(stream); // Drop the stream explicitly if needed
    });

    (receiver, thread_handle)
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("An error occurred on the audio stream: {}", err);
}

// audio_capture_startup_system revised to avoid premature removal of AudioReceiver
pub fn audio_capture_startup_system(
    mut commands: Commands,
    mut audio_receiver_res: Option<ResMut<AudioReceiver>>,
    visualization_type: Res<VisualizationType>,
    audio_thread_flag: Option<Res<AudioThreadFlag>>,
    config: Res<CfgResource>,
) {
    if visualization_type.is_changed() {
        // Signal the audio thread to stop
        if let Some(flag) = audio_thread_flag {
            flag.0.store(false, Ordering::SeqCst);
        }

        // Join the audio thread
        if let Some(mut receiver) = audio_receiver_res {
            if let Some(thread_handle) = receiver.thread_handle.take() {
                match thread_handle.join() {
                    Ok(_) => println!("Audio thread joined successfully."),
                    Err(e) => eprintln!("Failed to join audio thread: {:?}", e),
                }
            }

            // Now it's safe to remove the AudioReceiver resource
            commands.remove_resource::<AudioReceiver>();
        }

        // Restart the audio thread with a new run flag
        let new_run_flag = Arc::new(AtomicBool::new(true));

        let (audio_receiver, thread_handle) = if config.0.mic_mode {
            // Pass the run flag to the stream_input function
            stream_input(DeviceType::Input, new_run_flag.clone(), &config.0)
        } else {
            // Pass the run flag to the stream_input function
            stream_input(DeviceType::Output, new_run_flag.clone(), &config.0)
        };

        commands.insert_resource(AudioThreadFlag(new_run_flag));
        commands.insert_resource(AudioReceiver {
            receiver: Arc::new(Mutex::new(audio_receiver)),
            thread_handle: Some(thread_handle),
        });
    }
}
