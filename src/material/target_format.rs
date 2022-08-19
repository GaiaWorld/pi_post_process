
pub const ERROR_NOT_SUPPORT_TARGET_FORMAT: &str = "NOT_SUPPORT_TARGET_FORMAT";

pub const MOVE_E_TARGET_FORMAT: u128 = 10;

#[derive(Debug, Copy, Clone)]
pub enum ETexutureFormat {
    Rgba8UnormSrgb = 0,
    Rgba8Unorm,
    Bgra8UnormSrgb,
    Bgra8Unorm,
}

pub fn get_target_texture_format(
    e: ETexutureFormat
) -> wgpu::TextureFormat {
    match e {
        ETexutureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        ETexutureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        ETexutureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        ETexutureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
    }
}

pub fn as_target_texture_format(
    e: wgpu::TextureFormat
) -> Result<ETexutureFormat, String> {
    match e {
        wgpu::TextureFormat::Rgba8UnormSrgb => Ok(ETexutureFormat::Rgba8UnormSrgb),
        wgpu::TextureFormat::Rgba8Unorm => Ok(ETexutureFormat::Rgba8Unorm),
        wgpu::TextureFormat::Bgra8UnormSrgb => Ok(ETexutureFormat::Bgra8UnormSrgb),
        wgpu::TextureFormat::Bgra8Unorm => Ok(ETexutureFormat::Bgra8Unorm),
        _ => Err(String::from(ERROR_NOT_SUPPORT_TARGET_FORMAT)),
    }
}