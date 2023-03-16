#[derive(Clone, Copy, Debug)]
/// 颜色变换
pub struct HSB {
    /// [-180, 180]
    pub hue: i16,
    /// [-100, 100]
    pub saturate: i8,
    /// [-100, 100]
    pub brightness: i8,
}

impl Default for HSB {
    fn default() -> Self {
        Self {
            hue: 0,
            saturate: 0,
            brightness: 0,
        }
    }
}


impl HSB {
    pub fn is_enabled(
        &self
    ) -> bool {
        !(self.hue == 0 && self.saturate == 0 && self.brightness == 0)
    }
    ///
    /// F32x4
    pub fn collect(item: Option<&Self>, list: &mut Vec<f32>) {
        if let Some(item) = item {
            list.push(1.);
            list.push(item.hue as f32 / 360.);
            list.push(item.saturate as f32 / 100.);
            list.push(item.brightness as f32 / 100.);
        } else {
            list.push(0.);
            list.push(0.);
            list.push(0.);
            list.push(0.);
        }
    }
}