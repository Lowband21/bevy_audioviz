# Readme for Audio Visualizer Project

## Description

This project is a real-time audio visualizer implemented in Rust using the Bevy game engine and the CPAL library for audio processing. It captures audio data, performs a Fourier transform on it using the `rustfft` library to analyze frequencies, and then visualizes this data on a 2D canvas in a Bevy application.

![screenshot](https://raw.githubusercontent.com/Lowband21/bevy_audioviz/master/assets/screenshot.png)

## Features

- Real-time audio capture from the default input device.
- Fast Fourier Transform (FFT) of audio data to analyze frequency components.
- Visualization of audio frequency data on a dynamic 2D canvas.
- Smooth decay and interpolation of visualized data for aesthetic effect.
- Automatic adjustment of visualization to window resizing events.

## Prerequisites

Before you begin, ensure you have met the following requirements:

- Rust programming language installed.
- Cargo package manager for Rust.
- Bevy engine dependencies set up according to the [official Bevy setup guide](https://bevyengine.org/learn/book/getting-started/setup/).
- CPAL and rustfft library dependencies in your `Cargo.toml`.

## Setup

To set up the project, follow these steps:

1. Clone the repository to your local machine.
2. Navigate into the project directory.
3. Run `cargo build` to compile the project.

## Usage

To run the audio visualizer:

1. Execute `cargo run` from the terminal in the project directory.
2. Ensure that your audio input device is connected and recognized by the system.
3. Observe the visualized audio data in the application window.

## Components

The project consists of the following main components:

- `AudioResource`: A thread-safe resource holding the audio data buffer.
- `AudioMaterial`: A custom material that holds the normalized audio data for rendering.
- `AudioEntity`: A Bevy entity that holds the visual representation of the audio data.
- `AudioReceiver`: A wrapper around `mpsc::Receiver` to receive audio data events in the Bevy app.

## Systems

The following Bevy systems are used:

- `setup`: Initializes the 2D camera and other necessary startup configurations.
- `update_uniforms`: Updates the uniforms for the audio material based on the latest audio data.
- `spawn_visualization`: Spawns the visual elements for audio visualization.
- `window_resized_event`: Handles window resize events and adjusts the visualizer accordingly.
- `audio_event_system`: Processes audio events and updates the visualization state.
- `audio_capture_startup_system`: Initializes audio capture.

## Custom Shaders

The project includes custom shaders written in WGSL for rendering the audio visualization. Ensure the shader files are correctly placed in the `shaders` directory.

## Contributing

Contributions to this project are welcome. Please adhere to the project's coding standards and submit a pull request for review.

## License

Specify the type of license under which this project is released, for example, MIT or Apache 2.0.