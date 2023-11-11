use bevy::prelude::*;

use crate::BarMaterial;
use crate::CircleSplitMaterial;
use crate::PolygonMaterial;
use crate::WaveMaterial;

use crate::CfgResource;
use crate::GUIToggle;
use crate::StringMaterial;
//use bevy::math::Vec4Swizzles;
use bevy_egui::{egui, EguiContexts};

#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, uniform_update_ui_system);
    }
}

#[derive(Resource)]
pub struct Colors {
    pub monochrome: bool,
    pub color_start: Color,
    pub color_middle: Color,
    pub color_end: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            monochrome: true,
            color_start: Color::Rgba {
                red: 0.0,
                green: 0.0,
                blue: 6.66,
                alpha: 0.4,
            },
            color_middle: Color::Rgba {
                red: 0.0,
                green: 0.12,
                blue: 0.49,
                alpha: 0.2,
            },
            color_end: Color::Rgba {
                red: 0.29,
                green: 0.0,
                blue: 1.0,
                alpha: 1.0,
            },
        }
    }
}

fn uniform_update_ui_system(
    mut ctx: EguiContexts,
    mut bar_material: ResMut<Assets<BarMaterial>>,
    mut circle_split_material: ResMut<Assets<CircleSplitMaterial>>,
    mut string_material: ResMut<Assets<StringMaterial>>,
    mut wave_material: ResMut<Assets<WaveMaterial>>,
    mut polygon_material: ResMut<Assets<PolygonMaterial>>,
    mut colors: ResMut<Colors>, // Added the Colors resource
    mut config: ResMut<CfgResource>,
    toggle: Res<GUIToggle>,
) {
    // If the toggle is not active, return early
    if !toggle.active {
        return;
    }
    let context = ctx.ctx_mut();
    egui::Window::new("Update Uniforms").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.label("Interpolation Factor:");
            ui.add(egui::Slider::new(
                &mut config.0.interpolation_factor,
                0.0..=1.0,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Smoothing:");
            ui.add(egui::Slider::new(&mut config.0.smoothing, 0..=10));
        });
        ui.horizontal(|ui| {
            ui.label("Smoothing Size:");
            ui.add(egui::Slider::new(&mut config.0.smoothing_size, 1..=10));
        });
        ui.horizontal(|ui| {
            ui.label("Gate Threshold:");
            ui.add(egui::Slider::new(&mut config.0.gate_threshold, 0.0..=10.0));
        });

        // Determine the adjusted minimum and maximum values for the sliders
        let adjusted_freq_min_max = if config.0.frequency_min + 512.0 > config.0.frequency_max {
            config.0.frequency_min // If the max is too low, keep the min and adjust the max later
        } else {
            config.0.frequency_max - 512.0 // Otherwise, set the max limit for the min slider
        };

        let adjusted_freq_max_min = if config.0.frequency_max < config.0.frequency_min + 512.0 {
            config.0.frequency_max // If the min is too high, keep the max and adjust the min later
        } else {
            config.0.frequency_min + 512.0 // Otherwise, set the min limit for the max slider
        };

        // Draw the slider for frequency_min
        ui.horizontal(|ui| {
            ui.label("Frequency Min:");
            ui.add(egui::Slider::new(
                &mut config.0.frequency_min,
                20.0..=adjusted_freq_min_max,
            ));
        });

        // Draw the slider for frequency_max
        ui.horizontal(|ui| {
            ui.label("Frequency Max:");
            ui.add(egui::Slider::new(
                &mut config.0.frequency_max,
                adjusted_freq_max_min..=22_000.0,
            ));
        });

        // UI for updating the shared Colors resource
        ui.horizontal(|ui| {
            ui.label("Monochrome:");
            ui.checkbox(&mut colors.monochrome, "");
        });

        let mut color_start_arr = colors.color_start.into();
        let mut color_middle_arr = colors.color_middle.into();
        let mut color_end_arr = colors.color_end.into();

        ui.horizontal(|ui| {
            ui.label("Color Start:");
            if ui
                .color_edit_button_rgba_unmultiplied(&mut color_start_arr)
                .changed()
            {
                colors.color_start = Color::from(color_start_arr);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Color Middle:");
            if ui
                .color_edit_button_rgba_unmultiplied(&mut color_middle_arr)
                .changed()
            {
                colors.color_middle = Color::from(color_middle_arr);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Color End:");
            if ui
                .color_edit_button_rgba_premultiplied(&mut color_end_arr)
                .changed()
            {
                colors.color_end = Color::from(color_end_arr);
            }
        });

        if let Some(material) = bar_material.iter_mut().next() {
            material.1.monochrome = if colors.monochrome { 1 } else { 0 };
            material.1.colors[0] = colors.color_start.into();
            material.1.colors[1] = colors.color_middle.into();
            material.1.colors[2] = colors.color_end.into();
        }
        if let Some(material) = circle_split_material.iter_mut().next() {
            material.1.monochrome = if colors.monochrome { 1 } else { 0 };
            material.1.colors[0] = colors.color_start.into();
            material.1.colors[1] = colors.color_middle.into();
            material.1.colors[2] = colors.color_end.into();
        }
        if let Some(material) = string_material.iter_mut().next() {
            material.1.monochrome = if colors.monochrome { 1 } else { 0 };
            material.1.colors[0] = colors.color_start.into();
            material.1.colors[1] = colors.color_middle.into();
            material.1.colors[2] = colors.color_end.into();
        }
        if let Some(material) = wave_material.iter_mut().next() {
            material.1.monochrome = if colors.monochrome { 1 } else { 0 };
            material.1.colors[0] = colors.color_start.into();
            material.1.colors[1] = colors.color_middle.into();
            material.1.colors[2] = colors.color_end.into();
        }
        if let Some(material) = polygon_material.iter_mut().next() {
            material.1.monochrome = if colors.monochrome { 1 } else { 0 };
            material.1.colors[0] = colors.color_start.into();
            material.1.colors[1] = colors.color_middle.into();
            material.1.colors[2] = colors.color_end.into();
        }
    });
}
