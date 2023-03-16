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
    /// F32x6
    pub fn collect(item: Option<&Self>, list: &mut Vec<f32>) {
        if let Some(item) = item {
            list.push(1.);
            list.push(item.shadow_in as f32 / 255.);
            list.push(item.shadow_out as f32 / 255.);
            list.push(item.mid);
            list.push(item.highlight_in as f32 / 255.);
            list.push(item.highlight_out as f32 / 255.);
        } else {
            list.push(0.);
            list.push(0.);
            list.push(0.);
            list.push(0.);
            list.push(0.);
            list.push(0.);
        }
    }
}