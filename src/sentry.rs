pub struct Enemy {
    pub(crate) id: String,
    pub(crate) health: f32,
    pub(crate) position: (i32, i32),
    pub(crate) last_rotation: f32,
    pub(crate) speed: u32,
}
