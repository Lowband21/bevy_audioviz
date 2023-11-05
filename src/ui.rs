
use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};
use crate::bar_material::AudioMaterial;
use crate::circle_material::CircleMaterial;
use crate::polygon_material::PolygonMaterial;
use crate::circle_split_material::CircleSplitMaterial;

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
) {
    let context = ctx.ctx_mut();
    egui::Window::new("Update Uniforms").show(context, |ui| {
        if let Some(material) = circle_split_material.iter_mut().next() {
            // Directly assign the boolean value to `monochrome` based on the comparison
            let mut monochrome = material.1.monochrome == 1;

            // Display a checkbox in the UI that allows the user to modify `monochrome`
            ui.checkbox(&mut monochrome, "Monochrome");

            // Assign `material.1.monochrome` based on the state of `monochrome`
            material.1.monochrome = if monochrome { 1 } else { 0 };

            // Color editor for `color_start`
            //let mut color_start = [material.1.color_start[0], material.1.color_start[1], material.1.color_start[2]];
            //ui.color_edit_button_rgb(&mut color_start);
            //material.1.color_start = [color_start[0], color_start[1], color_start[2], 1.0];

            //// Color editor for `color_middle`
            //ui.color_edit_button_rgb(&mut material.1.color_middle);

            //// Color editor for `color_end`
            //ui.color_edit_button_rgb(&mut material.1.color_end);
        }
    });
}