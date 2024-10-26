use ggez::{
    glam::Vec2,
    graphics::{Canvas, Color, DrawParam, Mesh, MeshBuilder},
    Context,
};
#[derive(Clone)]
pub struct Task {
    pub(crate) task_times: Vec<f32>,
    pub(crate) goals: Vec<(f32, f32)>,
}
#[derive(Clone)]
pub struct Worker {
    pub(crate) health: f32,
    pub(crate) position: (i32, i32),
    pub(crate) speed: u32,
    pub(crate) path: Vec<(i32, i32)>,
    pub(crate) task: Task,
    pub(crate) time_since_path_started: f32,
    pub(crate) avalible_for_task: bool,
    pub(crate) ready_for_new_path: bool,
}
impl Worker {
    pub fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let current_time = ctx.time.time_since_start().as_secs_f32();
        let time_dif = current_time - self.time_since_path_started;
        let path_index: usize = (time_dif * self.speed as f32) as usize;
        if path_index < self.path.len() {
            self.position = self.path.get(path_index).unwrap().to_owned();
        } else if self.path.len() as f32 / self.speed as f32 + self.task.task_times[0] > time_dif {
            //wait the time needed
        } else {
            self.ready_for_new_path = true;
        }
        let mut mesh_builder = MeshBuilder::new();
        mesh_builder
            .circle(
                ggez::graphics::DrawMode::fill(),
                [self.position.0 as f32, self.position.1 as f32],
                5.,
                0.1,
                Color::BLUE,
            )
            .unwrap();
        let mesh_data = mesh_builder.build();
        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);
        canvas.draw(&mesh, DrawParam::default());
    }
}
