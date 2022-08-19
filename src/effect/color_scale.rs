/// 色阶
#[derive(Clone, Copy, Debug)]
pub struct ColorScale {
    /// Shadow In - [0, 255]
    pub shadow_in: u8,
    /// Shadow Out - [0, 255]
    pub shadow_out: u8,
    /// Mid - [0.1, 9.9]
    pub mid: f32,
    /// Highlight In - [0, 255]
    pub highlight_in: u8,
    /// Highlight Out - [0, 255]
    pub highlight_out: u8,
}

impl Default for ColorScale {
    fn default() -> Self {
        Self {
            shadow_in: 0,
            shadow_out: 0,
            mid: 1.,
            highlight_in: 255,
            highlight_out: 255,
        }
    }
}

impl ColorScale {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.shadow_in != 0 || self.shadow_out != 0 || self.mid != 1.0 || self.highlight_in != 255 || self.highlight_out != 255
    }
}