

// pub const SIMPLE_RENDER_EXTEND_FLOAT_COUNT: u16 = 2;
// pub const VERTEX_MATERIX_SIZE: u64 = (16 + (SIMPLE_RENDER_EXTEND_FLOAT_COUNT / 4 + 1) * 4) as u64 * 4;
// pub const DIFFUSE_MATERIX_SIZE: u64 = 4 * 4;

// pub struct TextureScaleOffset {
//     pub use_x: u32,
//     pub use_y: u32,
//     pub use_w: u32,
//     pub use_h: u32,
//     pub width: u32,
//     pub height: u32,
//     pub u_scale: f32,
//     pub v_scale: f32,
//     pub u_offset: f32,
//     pub v_offset: f32,
// }

// impl TextureScaleOffset {
//     pub fn from_rect(
//         use_x: u32,
//         use_y: u32,
//         use_w: u32,
//         use_h: u32,
//         width: u32,
//         height: u32,
//     ) -> Self {
//         let u_scale = width  as f32 / use_w as f32;
//         let v_scale = height as f32 / use_h as f32;
//         let u_offset = use_x as f32 / width  as f32;
//         let v_offset = use_y as f32 / height as f32;
        
//         Self { u_scale, v_scale, u_offset, v_offset, use_x, use_y, use_w, use_h, width, height }
//     }
// }

pub struct Shader {
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: wgpu::ShaderModule,
}

// /// 计划支持
// #[derive(Debug, Copy, Clone)]
// pub enum EPostprocessShader {
//     CopyIntensity = 1,
//     ColorEffect,
//     BlurDual,
//     BlurBokeh,
//     BlurRadial,
//     BlurDirect,
//     HorizonGlitch,
//     FilterBrightness,
//     Sobel,
//     RadialWave
// }

// pub fn blend_one_one() -> wgpu::BlendState {
//     wgpu::BlendState {
//         color: wgpu::BlendComponent {
//             src_factor: wgpu::BlendFactor::One,
//             dst_factor: wgpu::BlendFactor::One,
//             operation: wgpu::BlendOperation::Add,
//         },
//         alpha: wgpu::BlendComponent::OVER,
//     }
// }

// pub fn blend_one_zero() -> wgpu::BlendState {
//     wgpu::BlendState {
//         color: wgpu::BlendComponent {
//             src_factor: wgpu::BlendFactor::One,
//             dst_factor: wgpu::BlendFactor::Zero,
//             operation: wgpu::BlendOperation::Add,
//         },
//         alpha: wgpu::BlendComponent::OVER,
//     }
// }

pub fn load_shader(
    device: &wgpu::Device,
    vs_text: &str,
    fs_text: &str,
    vs_label: &str,
    fs_label: &str,
) -> Shader {
    let vs_module = device.create_shader_module(
        wgpu::ShaderModuleDescriptor {
            label: Some(vs_label),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(vs_text),
                stage: naga::ShaderStage::Vertex,
                defines: &[],
            }
        }
    );

    let fs_module = device.create_shader_module(
        wgpu::ShaderModuleDescriptor {
            label: Some(fs_label),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(fs_text),
                stage: naga::ShaderStage::Fragment,
                defines: &[],
            }
        }
    );

    Shader {
        vs_module,
        fs_module
    }
}