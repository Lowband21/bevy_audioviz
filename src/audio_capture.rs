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
use std::process::Command;
use std::time::Duration;

use crate::visualization::VisualizationType;

#[derive(Event, Debug)]
pub struct AudioProcessedEvent {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

#[derive(Debug, PartialEq)]
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
        let host = cpal::default_host();

        let devices = match device_type {
            DeviceType::Input => host.input_devices().expect("No default input device"),
            DeviceType::Output => host.output_devices().expect("No default output device"),
        };

        let mut device = host.default_output_device().unwrap();
        println!("Placeholder");

        if let Some(configured_device) = config.device.clone() {
            for (_device_index, dev) in devices.enumerate() {
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
                            let _buffer: Vec<f32> = data.to_vec();
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
                                rf_closure.store(false, Ordering::SeqCst); // Signal the thread to exit// Exit early to avoid further processing
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

        // Start monitoring the stream after the audio stream starts
        #[cfg(target_os = "linux")]
        {
            set_monitor_for_bevy_audioviz();
        }

        // Loop until the run flag is set to false.
        while run_flag.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(1));
        }

        // Perform any necessary cleanup here, if required
        drop(stream); // Drop the stream explicitly if needed
    });

    (receiver, thread_handle)
}

#[cfg(target_os = "linux")]
fn set_monitor_for_bevy_audioviz() {
    const MAX_ATTEMPTS: usize = 10;
    const DELAY_MS: u64 = 100; // Delay between attempts in milliseconds

    let mut attempts = 0;
    let mut bevy_audioviz_stream_id = String::new();

    while attempts < MAX_ATTEMPTS {
        // Step 1: List all source-outputs
        let output = Command::new("pactl")
            .arg("list")
            .arg("source-outputs")
            .output()
            .expect("Failed to execute pactl list source-outputs command");

        let source_outputs = String::from_utf8_lossy(&output.stdout);

        // Step 2: Parse the output in blocks to find the correct source-output ID
        for block in source_outputs.split("\n\n") {
            if block.contains("application.name = \"PipeWire ALSA [bevy_audioviz]\"") {
                for line in block.lines() {
                    if line.starts_with("Source Output #") {
                        bevy_audioviz_stream_id = line.split_whitespace().nth(2).unwrap_or("").trim_start_matches('#').to_string();
                        break;
                    }
                }
                break;
            }
        }

        if !bevy_audioviz_stream_id.is_empty() {
            break;
        }

        attempts += 1;
        thread::sleep(Duration::from_millis(DELAY_MS));
    }

    if bevy_audioviz_stream_id.is_empty() {
        eprintln!("Failed to find bevy_audioviz stream after {} attempts.", attempts);
        return;
    }

    let sink_output = Command::new("bash")
        .arg("-c")
        .arg("pactl get-default-sink")
        .output()
        .expect("Failed to execute pactl get-default-sink command");

    let default_sink = String::from_utf8_lossy(&sink_output.stdout).trim().to_string();

    if default_sink.is_empty() {
        eprintln!("Failed to find default sink.");
        return;
    }

    let monitor_source_output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "pactl list short sources | grep {}.monitor | awk '{{print $2}}'",
            default_sink
        ))
        .output()
        .expect("Failed to execute pactl list short sources command");

    let monitor_source = String::from_utf8_lossy(&monitor_source_output.stdout).trim().to_string();

    if monitor_source.is_empty() {
        eprintln!("Failed to find monitor source for default sink: {}", default_sink);
        return;
    }

    let move_output = Command::new("pactl")
        .arg("move-source-output")
        .arg(bevy_audioviz_stream_id.clone())
        .arg(monitor_source.clone())
        .output()
        .expect("Failed to move bevy_audioviz stream to monitor source");
    println!("pactl move-source-output {} {}", bevy_audioviz_stream_id, monitor_source.clone());

    if move_output.status.success() {
        println!("Set bevy_audioviz capture device to: {}", monitor_source);
    } else {
        eprintln!(
            "Failed to move source output: {}",
            String::from_utf8_lossy(&move_output.stderr)
        );
    }
}


fn err_fn(err: cpal::StreamError) {
    eprintln!("An error occurred on the audio stream: {}", err);
}

// audio_capture_startup_system revised to avoid premature removal of AudioReceiver
pub fn audio_capture_startup_system(
    mut commands: Commands,
    audio_receiver_res: Option<ResMut<AudioReceiver>>,
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
