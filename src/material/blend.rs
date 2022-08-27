

/// 计划支持 16 个设置 - 2^4
#[derive(Debug, Copy, Clone)]
pub enum EBlend {
    None = 0,
    // SrcAlpha , OneMinusSrcAlpha
    Combine = 1,
    // One, One
    Add = 2,
    // One, OneMinusSrcAlpha
    Premultiply = 3,
}

pub const MOVE_E_BLEND: u128 = 16;

pub fn get_blend_state(
    e: EBlend
) -> Option<wgpu::BlendState> {
    match e {
        EBlend::None => {
            None
        },
        EBlend::Combine => {
            Some(
                wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }
            )
        },
        EBlend::Add => {
            Some(
                wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }
            )
        },
        EBlend::Premultiply => {
            Some(
                wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }
            )
        },
    }
}