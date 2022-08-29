use crate::geometry::{glitch_geometry, image_effect_geometry, Geometry};

pub struct PostProcessGeometryManager {
    geometry: Option<Geometry>,
    glitch_geometry: Option<Geometry>,
}

impl Default for PostProcessGeometryManager {
    fn default() -> Self {
        Self {
            geometry: None,
            glitch_geometry: None,
        }
    }

}

impl PostProcessGeometryManager {
    pub fn new() -> Self {
        Self {
            geometry: None,
            glitch_geometry: None,
        }
    }

    pub fn check_geometry(
        &mut self,
        device: &wgpu::Device,
    ) -> &Geometry {
        if self.geometry.is_none() {
            self.geometry = Some(
                image_effect_geometry::create_geometry(device)
            );
        }

        self.geometry.as_ref().unwrap()
    }
    pub fn check_glitch_geometry(
        &mut self,
        device: &wgpu::Device,
    ) -> &Geometry {
        self.check_geometry(device);
        if self.glitch_geometry.is_none() {
            self.glitch_geometry = Some(
                glitch_geometry::create_geometry(device)
            );
        }

        self.glitch_geometry.as_ref().unwrap()
    }

    pub fn get_geometry(
        &self,
    ) -> &Geometry {
        self.geometry.as_ref().unwrap()
    }
    pub fn get_glitch_geometry(
        &self,
    ) -> &Geometry {
        self.glitch_geometry.as_ref().unwrap()
    }
}
