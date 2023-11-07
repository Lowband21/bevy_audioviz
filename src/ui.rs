use bevy::prelude::*;

use crate::bar_material::AudioMaterial;
use crate::circle_split_material::CircleSplitMaterial;

use crate::string_material::StringMaterial;
use crate::CfgResource;
use crate::GUIToggle;
use bevy::math::Vec4Swizzles;
use bevy_egui::{egui, EguiContexts};

#[derive(Default)]
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, uniform_update_ui_system);
    }
}

fn uniform_update_ui_system(
    mut ctx: EguiContexts,
    mut bar_material: ResMut<Assets<AudioMaterial>>,
    mut circle_split_material: ResMut<Assets<CircleSplitMaterial>>,
    mut string_material: ResMut<Assets<StringMaterial>>,
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
        let freq_max = config.0.frequency_max;
        ui.horizontal(|ui| {
            ui.label("Frequency Min:");
            ui.add(egui::Slider::new(
                &mut config.0.frequency_min,
                20.0..=freq_max - 100.0,
            ));
        });
        let freq_min = config.0.frequency_min;
        ui.horizontal(|ui| {
            ui.label("Frequency Max:");
            ui.add(egui::Slider::new(
                &mut config.0.frequency_max,
                freq_min + 100.0..=22_000.,
            ));
        });

        if let Some(material) = bar_material.iter_mut().next() {
            // Directly assign the boolean value to `monochrome` based on the comparison
            let mut monochrome = material.1.monochrome == 1;

            // Display a checkbox in the UI that allows the user to modify `monochrome`
            ui.checkbox(&mut monochrome, "Monochrome");

            // Assign `material.1.monochrome` based on the state of `monochrome`
            material.1.monochrome = if monochrome { 1 } else { 0 };

            // Color editor for `color_start`
            let mut color_start = material.1.colors[0].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_start);
            material.1.colors[0] = Vec4::new(color_start[0], color_start[1], color_start[2], 1.0);

            //// Color editor for `color_middle`
            let mut color_middle = material.1.colors[1].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_middle);
            material.1.colors[1] =
                Vec4::new(color_middle[0], color_middle[1], color_middle[2], 1.0);

            //// Color editor for `color_end`
            let mut color_end = material.1.colors[2].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_end);
            material.1.colors[2] = Vec4::new(color_end[0], color_end[1], color_end[2], 1.0);
        }
        if let Some(material) = circle_split_material.iter_mut().next() {
            // Directly assign the boolean value to `monochrome` based on the comparison
            let mut monochrome = material.1.monochrome == 1;

            // Display a checkbox in the UI that allows the user to modify `monochrome`
            ui.checkbox(&mut monochrome, "Monochrome");

            // Assign `material.1.monochrome` based on the state of `monochrome`
            material.1.monochrome = if monochrome { 1 } else { 0 };

            // Color editor for `color_start`
            let mut color_start = material.1.colors[0].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_start);
            material.1.colors[0] = Vec4::new(color_start[0], color_start[1], color_start[2], 1.0);

            //// Color editor for `color_middle`
            let mut color_middle = material.1.colors[1].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_middle);
            material.1.colors[1] =
                Vec4::new(color_middle[0], color_middle[1], color_middle[2], 1.0);

            //// Color editor for `color_end`
            let mut color_end = material.1.colors[2].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_end);
            material.1.colors[2] = Vec4::new(color_end[0], color_end[1], color_end[2], 1.0);
        }
        if let Some(material) = string_material.iter_mut().next() {
            // Directly assign the boolean value to `monochrome` based on the comparison
            let mut monochrome = material.1.monochrome == 1;

            // Display a checkbox in the UI that allows the user to modify `monochrome`
            ui.checkbox(&mut monochrome, "Monochrome");

            // Assign `material.1.monochrome` based on the state of `monochrome`
            material.1.monochrome = if monochrome { 1 } else { 0 };

            // Color editor for `color_start`
            let mut color_start = material.1.colors[0].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_start);
            material.1.colors[0] = Vec4::new(color_start[0], color_start[1], color_start[2], 1.0);

            //// Color editor for `color_middle`
            let mut color_middle = material.1.colors[1].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_middle);
            material.1.colors[1] =
                Vec4::new(color_middle[0], color_middle[1], color_middle[2], 1.0);

            //// Color editor for `color_end`
            let mut color_end = material.1.colors[2].xyz().to_array();
            ui.color_edit_button_rgb(&mut color_end);
            material.1.colors[2] = Vec4::new(color_end[0], color_end[1], color_end[2], 1.0);
        }
    });
}
