/// 色彩平衡
#[derive(Clone, Copy, Debug)]
pub struct ColorBalance {
    /// R channel
    pub r: u8,
    /// G channel
    pub g: u8,
    /// B channel
    pub b: u8,
}

impl Default for ColorBalance {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

impl ColorBalance {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.r != 255 || self.g != 255 || self.b != 255
    }
}