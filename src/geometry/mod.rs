use pi_hash::XHashMap;

pub mod glitch_geometry;
pub mod image_effect_geometry;
pub mod vertex_buffer_layout;

pub const IDENTITY_MATRIX: [f32; 16] = [
    1., 0., 0., 0.,
    0., 1., 0., 0.,
    0., 0., 1., 0.,
    0., 0., 0., 1.
];

pub struct Geometry {
    pub vertex_datas: XHashMap<u16, Vec<f32>>,
    pub vertex_buffers: XHashMap<u16, wgpu::Buffer>,
    pub indices_records: XHashMap<u16, u32>,
    pub indices_datas: XHashMap<u16, Vec<f32>>,
    pub indices_buffers: XHashMap<u16, wgpu::Buffer>,
    pub vertex_attrs: XHashMap<u16, Vec<wgpu::VertexAttribute>>,
}

#[derive(Debug, Clone, Copy)]
pub enum EGeometryBuffer {
    Position2D = 0,
    Indices,
    GlitchInstance,
}

pub struct VertexPosition2DViewer;

impl VertexPosition2DViewer {

    pub const PER_POINT_VALUE_CONT: u16 = 2;
    pub fn count(data: &Vec<f32>) -> u16 {
        (data.len() / 2) as u16
    }
    pub fn point_size() -> u16 {
        VertexPosition2DViewer::PER_POINT_VALUE_CONT
    }
    pub fn point_byte() -> u16 {
        VertexPosition2DViewer::PER_POINT_VALUE_CONT * std::mem::size_of::<f32>() as u16
    }
    pub fn desc<'a>(
        attributes: &'a Vec<wgpu::VertexAttribute>
    ) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: VertexPosition2DViewer::point_byte() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: attributes,
        }
    }
}

/// 故障纹实例化Buffer
pub struct GlitchInstanceViewer;

impl GlitchInstanceViewer {
    pub const MAX_INSTANCE_COUNT: usize = 32;
    pub const DATA_COUNT: u16 = 4;
    pub fn bytes() -> u16 {
        GlitchInstanceViewer::DATA_COUNT * std::mem::size_of::<f32>() as u16
    }
    pub fn desc<'a>(
        attributes: &'a Vec<wgpu::VertexAttribute>
    ) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: GlitchInstanceViewer::bytes() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: attributes,
        }
    }
}

pub struct IndicesViewer;

impl IndicesViewer {
    pub const TRIANGLE_POINT_CONT: u16 = 3;
    pub fn count(data: &Vec<u16>) -> u16 {
        data.len() as u16
    }
    pub fn triangle_byte() -> u16 {
        IndicesViewer::TRIANGLE_POINT_CONT * std::mem::size_of::<u16>() as u16
    }
}