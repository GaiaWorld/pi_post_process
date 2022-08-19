pub struct TextureBlock {
    /// 使用在材质的哪个纹理槽位
    pub use_for_slot: u8,
    pub u_scale: f32,
    pub v_scale: f32,
    pub u_offset: f32,
    pub v_offset: f32,
}