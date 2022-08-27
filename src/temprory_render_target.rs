use std::num::NonZeroU32;

use guillotiere::Rectangle;
use pi_render::components::view::target_alloc::{ShareTargetView, SafeAtlasAllocator, TargetDescriptor, TextureDescriptor};
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy)]
pub struct PostprocessTexture<'a> {
    pub use_x: u32,
    pub use_y: u32,
    pub use_w: u32,
    pub use_h: u32,
    pub width: u32,
    pub height: u32,
    pub view: &'a wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

#[derive(Clone)]
pub struct PostprocessShareTarget {
    pub use_x: u32,
    pub use_y: u32,
    pub use_w: u32,
    pub use_h: u32,
    pub view: ShareTargetView,
    pub format: wgpu::TextureFormat,
}

#[derive(Clone)]
pub enum EPostprocessTarget<'a> {
    TextureView(PostprocessTexture<'a>),
    ShareTarget(PostprocessShareTarget),
}

#[derive(Clone)]
pub enum EPostprocessResult {
    Source,
    ShareTarget(PostprocessShareTarget),
}

impl EPostprocessResult {
    pub fn from_share_target(
        view: ShareTargetView,
        format: wgpu::TextureFormat,
    ) -> Self {
        let (use_x, use_y, use_w, use_h) = get_rect_info(view.rect());
        Self::ShareTarget(
            PostprocessShareTarget {
                use_x,
                use_y,
                use_w,
                use_h,
                view,
                format,
            }
        )
    }
}

impl<'a> EPostprocessTarget<'a> {
    pub fn from_share_target(
        view: ShareTargetView,
        format: wgpu::TextureFormat,
    ) -> Self {
        let (use_x, use_y, use_w, use_h) = get_rect_info(view.rect());
        Self::ShareTarget(
            PostprocessShareTarget {
                use_x,
                use_y,
                use_w,
                use_h,
                view,
                format,
            }
        )
    }
    pub fn use_x(&self) -> u32 {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.use_x
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.use_x
            },
        }
    }
    pub fn use_y(&self) -> u32 {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.use_y
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.use_y
            },
        }
    }
    pub fn use_w(&self) -> u32 {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.use_w
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.use_w
            },
        }
    }
    pub fn use_h(&self) -> u32 {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.use_h
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.use_h
            },
        }
    }
    pub fn width(&self) -> u32 {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.width
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.view.target().width
            },
        }
    }
    pub fn height(&self) -> u32 {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.height
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.view.target().height
            },
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.view
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.view.target().colors[0].0.as_ref()
            },
        }
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        match self {
            EPostprocessTarget::TextureView(value) => {
                value.format
            },
            EPostprocessTarget::ShareTarget(value) => {
                value.format
            },
        }
    }
    pub fn get_rect(&self) -> (u32, u32, u32, u32) {
        match self {
            EPostprocessTarget::TextureView(value) => {
                (value.use_x, value.use_x, value.use_w, value.use_h)
            },
            EPostprocessTarget::ShareTarget(value) => {
                (value.use_x, value.use_x, value.use_w, value.use_h)
            },
        }
    }
    pub fn get_full_size(&self) -> (u32, u32) {
        match self {
            EPostprocessTarget::TextureView(value) => {
                (value.width, value.height)
            },
            EPostprocessTarget::ShareTarget(value) => {
                let fbo = value.view.target();
                let width = fbo.width;
                let height = fbo.height;
                (width, height)
            },
        }
    }
    pub fn get_share_target(&self) -> Option<ShareTargetView> {
        match self {
            EPostprocessTarget::TextureView(value) => {
                None
            },
            EPostprocessTarget::ShareTarget(value) => {
                Some(value.view.clone())
            },
        }
    }
}

pub struct TemporaryRenderTargets<'a> {
    targets: Vec<Option<EPostprocessTarget<'a>>>,
    id_for_index: Vec<usize>,
    atlas_allocator: &'a SafeAtlasAllocator,
}

impl<'a> TemporaryRenderTargets<'a> {
    pub fn new(atlas_allocator: &'a SafeAtlasAllocator) -> Self {
        Self { targets: Vec::new(), id_for_index: vec![], atlas_allocator }
    }
    pub fn get_share_target_view(
        &mut self,
        id: Option<usize>,
    ) -> Option<ShareTargetView> {
        if let Some(id) = id {
            let index = self.id_for_index.get(id);
            if let Some(index) = index {
                let item = self.targets.get(*index);
                
                if let Some(Some(target)) = item {
                    target.get_share_target()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn record_from_other(
        &mut self,
        other: EPostprocessTarget<'a>,
    ) -> usize {
        let result = self.id_for_index.len();
    
        let index = self.targets.len();
        self.targets.push(Some(other));
        
        self.id_for_index.push(index);

        result
    }
    pub fn reset(
        &mut self,
    ) {
        self.id_for_index.clear();
        self.targets.clear();
    }
    pub fn create_share_target(
        &mut self,
        without_id: Option<usize>,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> usize {
        let mut without_list = vec![];

        if let Some(without_id) = without_id {
            let without_index = self.id_for_index.get(without_id);
            let without = self.targets.get(*without_index.unwrap()).unwrap();

            if let Some(without) = without {
                let target = without.get_share_target();
                if let Some(target) = target {
                    without_list.push(target);
                }
            };
        }

        let result = self.id_for_index.len();

        let index = self.targets.len();
        self.id_for_index.push(index);

        let view = get_share_target_view(
            self.atlas_allocator,
            width,
            height,
            format,
            &without_list
        );

        let (use_x, use_y, use_w, use_h) = get_rect_info(view.rect());

        let target = EPostprocessTarget::ShareTarget(
            PostprocessShareTarget { use_x, use_y, use_w, use_h, view, format, }
        );
        self.targets.push(
            Some(target)
        );

        result
    }
    pub fn release(
        &mut self,
        id: usize
    ) {
            let index = *self.id_for_index.get(id).unwrap();
            // self.targets.get_mut(index) = None;
            self.targets[index] = None;
    }
    pub fn release2(
        &mut self,
        id: Option<usize>
    ) {
        if let Some(id) = id {
            let index = *self.id_for_index.get(id).unwrap();
            // self.targets.get_mut(index) = None;
            self.targets[index] = None;
        }

    }
    pub fn get_view(
        &self,
        id: usize
    ) -> Option<&wgpu::TextureView> {
        let index = self.id_for_index.get(id).unwrap();
        let item = self.targets.get(*index);

        if let Some(Some(item)) = item {
            Some(item.view())
        } else {
            None
        }
    }
    pub fn get_rect(
        &self,
        id: usize
    ) -> Option<(u32, u32, u32, u32)> {
        let index = self.id_for_index.get(id).unwrap();
        let item = self.targets.get(*index);

        if let Some(Some(item)) = item {
            Some(item.get_rect())
        } else {
            None
        }
    }
    pub fn get_full_size(
        &self,
        id: usize
    ) -> Option<(u32, u32)> {
        let index = self.id_for_index.get(id).unwrap();
        let item = self.targets.get(*index);

        if let Some(Some(item)) = item {
            Some(item.get_full_size())
        } else {
            None
        }
    }
    pub fn get_format(
        &self,
        id: usize
    ) -> Option<wgpu::TextureFormat> {
        let index = self.id_for_index.get(id).unwrap();
        let item = self.targets.get(*index);

        if let Some(Some(item)) = item {
            Some(item.format())
        } else {
            None
        }
    }
    pub fn get_target(
        &self,
        id: usize
    ) -> Option<&EPostprocessTarget> {
        let index = self.id_for_index.get(id).unwrap();
        let item = self.targets.get(*index);

        if let Some(Some(item)) = item {
            Some(item)
        } else {
            None
        }
    }
    pub fn src_to_dst_isok(
        &self,
        src_id: Option<usize>,
        dst_id: Option<usize>,
    ) -> bool {
        if let (Some(src_id), Some(dst_id)) = (src_id, dst_id) {
            let index = *self.id_for_index.get(src_id).unwrap();
            let src = self.targets.get(index).unwrap();
            let index = *self.id_for_index.get(dst_id).unwrap();
            let dst = self.targets.get(index).unwrap();

            if let (Some(src), Some(dst)) = (src, dst) {
                return true;
                // *src.target() == *dst.target()
            };
        }
        true
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
            texture_descriptor: SmallVec::from_slice(
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
        }),
        temp_rendertarget_list.iter()
    );

    srt
}