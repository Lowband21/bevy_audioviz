# Bevy Audioviz

## Description
This project is a real-time audio visualizer implemented in Rust using the Bevy game engine and the CPAL library for audio processing. It captures audio data, performs frequency analysis using the `spectrum-analyzer` crate, and then visualizes this data on a 2D canvas in a Bevy application using shaders.

![screenshot bar](https://raw.githubusercontent.com/Lowband21/bevy_audioviz/master/assets/screenshot_bar.png)
![screenshot string](https://raw.githubusercontent.com/Lowband21/bevy_audioviz/master/assets/screenshot_string.png)
![screenshot circle split](https://raw.githubusercontent.com/Lowband21/bevy_audioviz/master/assets/screenshot_circle_split.png)

## New Features
- **GUI Integration**: Press `G` to toggle the new graphical user interface that allows you to adjust various visualization parameters including:
  - Colors
  - Monochrome effect
  - Frequency range (max and min)
  - Interpolation factor
  - Smoothing
  - Smoothing size
- **Multiple Visualizers**: Choose between three distinct visualizers:
  - Bar Visualizer
  - String Visualizer
  - Circle Visualizer

## Features
- Real-time audio capture from configurable input or output device.
- Extreme low latency: Less than 0.5ms per frame with vsync disabled.
- Spectrum analysis visualizer with a focus on perceived accuracy.
- Symmetric circle visualizer with separated channels.
- Configurable smooth decay and interpolation of visualized data for aesthetic effect.
- Automatic scaling of visualization to window resizing events.

## Building From Source:
To set up the project, follow these steps:
1. Clone the repository to your local machine.
2. Navigate into the project directory.
3. Run `cargo build` to compile the project.

## Usage
To run the audio visualizer:
1. Execute `cargo run` from the terminal in the project directory.
2. Ensure that your audio input device is connected and recognized by the system.
3. Play some audio.
4. Observe the visualized audio data in the application window.
5. Press `G` to toggle the GUI and adjust visualization settings.

### Keybindings:
| Key | Action |
| --- | ------ |
| Space | Switch Visualization |
| V | Toggle VSync |
| G | Toggle GUI |

## Custom Shaders

The project includes custom shaders written in WGSL for rendering the audio visualization. Ensure the shader files are correctly placed in the `assets/shaders` directory.

## Contributing

Contributions to this project are welcome. Submit a pull request for review.

## License
This project is released under the MIT license, see LICENSE for more details.

## Credits
I took a great deal of inspiration from Audioviz, created by BrunoWallner.
