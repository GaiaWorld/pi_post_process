


mod temprory_render_target;
// pub mod postprocess_geometry;
pub mod postprocess;
pub mod effect;
mod geometry;
mod material;
mod renderer;
// pub mod postprocess_pipeline;
// pub mod postprocess_renderer;
pub mod error;
pub mod image_effect;
pub mod prelude;

mod postprocess_flags;

pub const IDENTITY_MATRIX: [f32; 16] = [
    1., 0., 0., 0.,
    0., 1., 0., 0.,
    0., 0., 1., 0.,
    0., 0., 0., 1.
];


#[derive(Debug, Clone, Copy)]
pub struct SimpleRenderExtendsData {
    pub alpha: f32,
    pub depth: f32,
}

impl SimpleRenderExtendsData {
    pub fn default() -> Self {
        Self {
            alpha: 1.0,
            depth: 1.0,
        }
    }
}
