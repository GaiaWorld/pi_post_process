use pi_hash::XHashMap;
use wgpu::util::DeviceExt;

use crate::{geometry::{Geometry, GlitchInstanceViewer, EGeometryBuffer}, };

pub fn create_geometry(
    device: &wgpu::Device,
) -> Geometry {
    let mut vertex_usage = wgpu::BufferUsages::VERTEX;
    let mut indices_usage = wgpu::BufferUsages::INDEX;

    let vertex_data: Vec<f32> = vec![
        -0.5,   -0.5,
         0.5,   -0.5,
        -0.5,    0.5,
         0.5,    0.5,
    ];
    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertex_data),
            usage: vertex_usage,
        }
    );

    let indices_data: Vec<u16> = vec![
        0, 1, 2,
        2, 1, 3
    ];
    let indices_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices_data),
            usage: indices_usage,
        }
    );

    let instance_data: [f32; GlitchInstanceViewer::MAX_INSTANCE_COUNT * GlitchInstanceViewer::DATA_COUNT as usize] = [0.; GlitchInstanceViewer::MAX_INSTANCE_COUNT * GlitchInstanceViewer::DATA_COUNT as usize];
    let instance_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }
    );

    let mut geometry = Geometry {
        vertex_datas: XHashMap::default(),
        vertex_buffers: XHashMap::default(),
        indices_records: XHashMap::default(),
        indices_datas: XHashMap::default(),
        indices_buffers: XHashMap::default(),
        vertex_attrs: XHashMap::default(),
    };

    let key = EGeometryBuffer::Position2D;
    geometry.vertex_buffers.insert(key as u16, vertex_buffer);
    geometry.vertex_attrs.insert(key as u16, vec![
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0
        }
    ]);
    
    let key = EGeometryBuffer::GlitchInstance;
    geometry.vertex_buffers.insert(key as u16, instance_buffer);
    geometry.vertex_attrs.insert(key as u16, vec![
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 1
        }
    ]);
    
    let key = EGeometryBuffer::Indices;
    geometry.indices_buffers.insert(key as u16, indices_buffer);
    geometry.indices_records.insert(key as u16, indices_data.len() as u32);

    geometry
}