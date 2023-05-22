

use guillotiere::Rectangle;
use pi_render::{components::view::target_alloc::{ShareTargetView, SafeAtlasAllocator, TargetDescriptor, TextureDescriptor}, renderer::texture::ETextureViewUsage};
use smallvec::SmallVec;

#[derive(Clone)]
pub struct PostprocessTexture {
    pub use_x: u32,
    pub use_y: u32,
    pub use_w: u32,
    pub use_h: u32,
    pub width: u32,
    pub height: u32,
    pub view: ETextureViewUsage,
    pub format: wgpu::TextureFormat,
}

impl PostprocessTexture {
    pub fn from_share_target(
        view: ShareTargetView,
        format: wgpu::TextureFormat,
    ) -> Self {
        let (use_x, use_y, use_w, use_h) = get_rect_info(view.rect());
        PostprocessTexture {
            use_x,
            use_y,
            use_w,
            use_h,
            width: view.target().width,
            height: view.target().height,
            view: ETextureViewUsage::SRT(view),
            format,
        }
    }
    pub fn use_x(&self) -> u32 {
        self.use_x
    }
    pub fn use_y(&self) -> u32 {
        self.use_y
    }
    pub fn use_w(&self) -> u32 {
        self.use_w
    }
    pub fn use_h(&self) -> u32 {
        self.use_h
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn view(&self) -> &wgpu::TextureView {
        self.view.view()
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format.clone()
    }
    pub fn get_rect(&self) -> (u32, u32, u32, u32) {
        (self.use_x, self.use_y, self.use_w, self.use_h)
    }
    pub fn get_tilloff(&self) -> (f32, f32, f32, f32) {
        (self.use_x as f32 / self.width as f32, self.use_y as f32 / self.height as f32, self.use_w as f32 / self.width as f32, self.use_h as f32 / self.height as f32)
    }
    pub fn get_full_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn get_share_target(&self) -> Option<ShareTargetView> {
        match &self.view {
            ETextureViewUsage::SRT(val) => Some(val.clone()),
            _ => None,
        }
    }
    pub fn size_eq(&self, rhs: &Self) -> bool {
        self.use_w == rhs.use_w && self.use_h == rhs.use_h
    }
    pub fn size_eq_2(&self, rhs: &(u32, u32)) -> bool {
        self.use_w == rhs.0 && self.use_h == rhs.1
    }
}

pub fn get_rect_info(rect: &Rectangle) -> (u32, u32, u32, u32) {
    (
        rect.min.x as u32,
        rect.min.y as u32,
        (rect.max.x - rect.min.x) as u32,
        (rect.max.y - rect.min.y) as u32,
    )
}

pub fn get_share_target_view(
    atlas_allocator: &SafeAtlasAllocator,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    temp_rendertarget_list: &Vec<ShareTargetView>
) -> ShareTargetView {

    // println!("get_share_target_view, f = {:?}", format);

    let srt = atlas_allocator.allocate(
        width,
        height,
        atlas_allocator.get_or_create_type(TargetDescriptor {
            colors_descriptor: SmallVec::from_slice(
                &[
                    TextureDescriptor {
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: format,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                        base_mip_level: 0,
                        base_array_layer: 0,
                        array_layer_count: None,
                        view_dimension: None,
                    }
                ]
            ),
            need_depth: false,
            default_width: width,
            default_height: height,
            depth_descriptor: None,
        }),
        temp_rendertarget_list.iter()
    );

    srt
}