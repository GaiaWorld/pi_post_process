#[derive(Clone, Copy, Debug)]
pub struct Alpha {
    ///不半透明度
    pub a: f32,
}

impl Default for Alpha {
    fn default() -> Self {
        Self {
            a: 1.0,
        }
    }
}