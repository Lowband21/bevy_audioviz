use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

use crate::ARRAY_UNIFORM_SIZE;

#[macro_export]
macro_rules! impl_material_new {
    ($material_type:ty) => {
        impl $material_type {
            pub fn new(width: f32, height: f32, colors: &Colors) -> Self {
                Self {
                    left_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); 16],
                    right_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); 16],
                    viewport_width: width,
                    viewport_height: height,
                    monochrome: if colors.monochrome { 1 } else { 0 },
                    colors: [
                        Vec4::new(
                            colors.color_start.r(),
                            colors.color_start.g(),
                            colors.color_start.b(),
                            colors.color_start.a(),
                        ),
                        Vec4::new(
                            colors.color_middle.r(),
                            colors.color_middle.g(),
                            colors.color_middle.b(),
                            colors.color_middle.a(),
                        ),
                        Vec4::new(
                            colors.color_end.r(),
                            colors.color_end.g(),
                            colors.color_end.b(),
                            colors.color_end.a(),
                        ),
                        Vec4::ZERO,
                    ],
                }
            }
        }
    };
}
#[macro_export]
macro_rules! prepare_material {
    ($material_type:ty, $materials:expr, $width:expr, $height:expr, $colors:expr) => {
        $materials.add(<$material_type>::new($width, $height, $colors))
    };
}

#[macro_export]
macro_rules! impl_one_channel_material_new {
    ($material_type:ty) => {
        impl $material_type {
            pub fn new(width: f32, height: f32, colors: &Colors) -> Self {
                Self {
                    normalized_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); 16],
                    viewport_width: width,
                    viewport_height: height,
                    monochrome: if colors.monochrome { 1 } else { 0 },
                    colors: [
                        Vec4::new(
                            colors.color_start.r(),
                            colors.color_start.g(),
                            colors.color_start.b(),
                            colors.color_start.a(),
                        ),
                        Vec4::new(
                            colors.color_middle.r(),
                            colors.color_middle.g(),
                            colors.color_middle.b(),
                            colors.color_middle.a(),
                        ),
                        Vec4::new(
                            colors.color_end.r(),
                            colors.color_end.g(),
                            colors.color_end.b(),
                            colors.color_end.a(),
                        ),
                        Vec4::ZERO,
                    ],
                }
            }
        }
    };
}

#[derive(Resource, Default)]
pub struct BarEntity(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct StringEntity(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct CircleSplitEntity(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct WaveEntity(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct PolygonEntity(pub Option<Entity>);

#[derive(Component, Debug, Clone, AsBindGroup, TypePath, Asset)]
pub struct StringMaterial {
    #[uniform(0)]
    pub left_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(1)]
    pub right_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(2)]
    pub viewport_width: f32,
    #[uniform(3)]
    pub viewport_height: f32,
    #[uniform(4)]
    pub monochrome: u32,
    #[uniform(5)]
    pub colors: [Vec4; 4],
}
impl Material2d for StringMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/string_fragment.wgsl".into()
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypePath, Asset)]
pub struct CircleSplitMaterial {
    #[uniform(0)]
    pub left_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(1)]
    pub right_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(2)]
    pub viewport_width: f32,
    #[uniform(3)]
    pub viewport_height: f32,
    #[uniform(4)]
    pub monochrome: u32,
    #[uniform(5)]
    pub colors: [Vec4; 4],
}
impl Material2d for CircleSplitMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/circle_split_fragment.wgsl".into()
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypePath, Asset)]
pub struct WaveMaterial {
    #[uniform(0)]
    pub left_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(1)]
    pub right_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(2)]
    pub viewport_width: f32,
    #[uniform(3)]
    pub viewport_height: f32,
    #[uniform(4)]
    pub monochrome: u32,
    #[uniform(5)]
    pub colors: [Vec4; 4],
}
impl Material2d for WaveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wave_fragment.wgsl".into()
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypePath, Asset)]
pub struct PolygonMaterial {
    #[uniform(0)]
    pub normalized_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(1)]
    pub viewport_width: f32,
    #[uniform(2)]
    pub viewport_height: f32,
    #[uniform(3)]
    pub monochrome: u32,
    #[uniform(4)]
    pub colors: [Vec4; 4],
}
impl Material2d for PolygonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/polygon_fragment.wgsl".into()
    }
}

#[derive(Component, Debug, Clone, AsBindGroup, TypePath, Asset)]
pub struct BarMaterial {
    #[uniform(0)]
    pub normalized_data: [Vec4; ARRAY_UNIFORM_SIZE],
    #[uniform(1)]
    pub viewport_width: f32,
    #[uniform(2)]
    pub viewport_height: f32,
    #[uniform(3)]
    pub monochrome: u32,
    #[uniform(4)]
    pub colors: [Vec4; 4],
}
impl Material2d for BarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bar_fragment.wgsl".into()
    }
}
