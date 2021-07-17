use ash::vk;
use model::{Model, SceneObject};
use openxr as xr;
use std::{collections::HashMap, io::Seek};

pub use app::App;
pub use hotham_error::HothamError;
pub use uniform_buffer_object::UniformBufferObject;
pub use vertex::Vertex;

mod app;
mod buffer;
mod camera;
mod frame;
mod hotham_error;
mod image;
pub mod model;
mod renderer;
mod swapchain;
mod texture;
mod uniform_buffer_object;
mod util;
mod vertex;
mod vulkan_context;
mod animation;
mod node;

pub type HothamResult<T> = std::result::Result<T, HothamError>;
pub const COLOR_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
pub const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
pub const VIEW_COUNT: u32 = 2;
pub const SWAPCHAIN_LENGTH: usize = 3;
pub const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;
pub const BLEND_MODE: xr::EnvironmentBlendMode = xr::EnvironmentBlendMode::OPAQUE;

#[cfg(target_os = "windows")]
pub const TEXTURE_FORMAT: vk::Format = vk::Format::BC7_SRGB_BLOCK;

#[cfg(target_os = "android")]
pub const TEXTURE_FORMAT: vk::Format = vk::Format::ASTC_4X4_SRGB_BLOCK;

pub trait Program {
    fn get_model_data(&self) -> (&[u8], &[u8]);
    fn init(&mut self, models: HashMap<String, Model>) -> HothamResult<Vec<SceneObject>>;
}

#[derive(Debug, Clone)]
pub struct ProgramInitialization<'a> {
    pub vertices: &'a Vec<Vertex>,
    pub indices: &'a Vec<u32>,
    pub vertex_shader: Vec<u32>,
    pub fragment_shader: Vec<u32>,
    pub image_buf: Vec<u8>,
    pub image_height: u32,
    pub image_width: u32,
}

pub fn read_spv_from_bytes<R: std::io::Read + Seek>(bytes: &mut R) -> std::io::Result<Vec<u32>> {
    ash::util::read_spv(bytes)
}
