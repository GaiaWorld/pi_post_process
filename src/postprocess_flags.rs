#[derive(Debug, Clone, Copy)]
pub struct PostprocessFlags {
    pub bloom_dual:         bool,
    pub blur_bokeh:         bool,
    pub blur_direct:        bool,
    pub blur_dual:          bool,
    pub blur_radial:        bool,
    pub color_effect:       bool,
    pub copy_intensity:     bool,
    pub filter_sobel:       bool,
    pub horizon_glitch:     bool,
    pub radial_wave:        bool,
    pub active_count:       u8,
}

impl Default for PostprocessFlags {
    fn default() -> Self {
        Self {
            bloom_dual: false,
            blur_bokeh: false,
            blur_direct: false,
            blur_dual: false,
            blur_radial: false,
            color_effect: false,
            copy_intensity: false,
            filter_sobel: false,
            horizon_glitch: false,
            radial_wave: false,
            active_count: 0
        }
    }
}