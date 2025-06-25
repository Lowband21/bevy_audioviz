use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::pbr::Material;

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
                        {
                            let srgba = colors.color_start.to_srgba();
                            Vec4::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
                        },
                        {
                            let srgba = colors.color_middle.to_srgba();
                            Vec4::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
                        },
                        {
                            let srgba = colors.color_end.to_srgba();
                            Vec4::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
                        },
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
                        {
                            let srgba = colors.color_start.to_srgba();
                            Vec4::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
                        },
                        {
                            let srgba = colors.color_middle.to_srgba();
                            Vec4::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
                        },
                        {
                            let srgba = colors.color_end.to_srgba();
                            Vec4::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
                        },
                        Vec4::ZERO,
                    ],
                }
            }
        }
    };
}

#[derive(Resource, Default)]
pub struct BarEntity {
    pub entity: Option<Entity>,
    pub material_handle: Option<Handle<BarMaterial>>,
}

#[derive(Resource, Default)]
pub struct StringEntity {
    pub entity: Option<Entity>,
    pub material_handle: Option<Handle<StringMaterial>>,
}

#[derive(Resource, Default)]
pub struct CircleSplitEntity {
    pub entity: Option<Entity>,
    pub material_handle: Option<Handle<CircleSplitMaterial>>,
}

#[derive(Resource, Default)]
pub struct WaveEntity {
    pub entity: Option<Entity>,
    pub material_handle: Option<Handle<WaveMaterial>>,
}

#[derive(Resource, Default)]
pub struct PolygonEntity {
    pub entity: Option<Entity>,
    pub material_handle: Option<Handle<PolygonMaterial>>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
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
impl Material for StringMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/string_fragment.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
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
impl Material for CircleSplitMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/circle_split_fragment.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
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
impl Material for WaveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wave_fragment.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
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
impl Material for PolygonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/polygon_fragment.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
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
impl Material for BarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bar_fragment.wgsl".into()
    }
}
