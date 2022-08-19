use crate::geometry::{VertexPosition2DViewer, GlitchInstanceViewer, Geometry, EGeometryBuffer};

#[derive(Debug, Copy, Clone)]
pub enum EVertexBufferLayout {
    Position2D = 0,
    Position2DGlitchInstance,
}

pub const MOVE_E_VERTEX_BUFFER_LAYOUT: u128 = 10;

pub trait GetVertexBufferLayouts {
    fn get_vertex_buffer_layouts(
        &self
    ) -> Vec<wgpu::VertexBufferLayout>;

    fn get_vertex_buffer_layouts_type(
        &self
    ) -> EVertexBufferLayout;
}

pub fn get_vertex_buffer_layouts<'a>(
    e: EVertexBufferLayout,
    geo: &'a Geometry
) -> Vec<wgpu::VertexBufferLayout<'a>> {
    match e {
        EVertexBufferLayout::Position2D => {
            vec![
                VertexPosition2DViewer::desc(
                    geo.vertex_attrs.get(&(EGeometryBuffer::Position2D as u16)).unwrap()
                )
            ]
        },
        EVertexBufferLayout::Position2DGlitchInstance => {
            vec![
                VertexPosition2DViewer::desc(
                    geo.vertex_attrs.get(&(EGeometryBuffer::Position2D as u16)).unwrap()
                ),
                GlitchInstanceViewer::desc(
                    geo.vertex_attrs.get(&(EGeometryBuffer::GlitchInstance as u16)).unwrap()
                )
            ]
        },
    }
}