use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh, MeshBuilder, Rect, Transform};
use ggez::mint::Point2;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use num::abs;
use std::f32::consts::PI;

const ROTATION_BOTTOM: f32 = 0.0;
const ROTATION_TOP: f32 = PI;
const ROTATION_TOP_LEFT: f32 = (3. * PI) / 4.;
const ROTATION_TOP_RIGHT: f32 = (5. * PI) / 4.;
const ROTATION_Bottom_LEFT: f32 = PI / 4.;
const ROTATION_Bottom_RIGHT: f32 = (PI * 7.) / 4.;
const ROTATION_RIGHT: f32 = (3. * PI) / 2.;
const ROTATION_LEFT: f32 = PI / 2.;
const STROKE_WIDTH: f32 = 2.0;
#[derive(Clone)]
pub struct Enemy {
    pub(crate) health: f32,
    pub(crate) position: (i32, i32),
    pub(crate) size: f32,
    pub(crate) path: Vec<(i32, i32)>,
    pub(crate) time_since_path_built: f32,
    pub(crate) last_rotation: f32,
    pub(crate) speed: u32,
}
pub struct Hitbox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) bottom_left: (f32, f32),
}
impl Enemy {
    pub fn draw_dead(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let mut mesh_builder = MeshBuilder::new();
        let orgin = (0.0 as f32, 0.0 as f32);
        let half_size = self.size / 2.;
        mesh_builder
            .polygon(
                graphics::DrawMode::stroke(STROKE_WIDTH),
                &[
                    [orgin.0 as f32, orgin.1 as f32 + self.size as f32],
                    [orgin.0 as f32 - half_size, orgin.1 as f32 - self.size],
                    [orgin.0 as f32 + half_size, orgin.1 as f32 - self.size],
                ],
                Color::from_rgb(139, 0, 0),
            )
            .unwrap();
        let mesh_data = mesh_builder.build();
        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);
        let current_position_point = Point2 {
            x: self.position.0 as f32,
            y: self.position.1 as f32,
        };
        canvas.draw(
            &mesh,
            DrawParam::default()
                .rotation(self.last_rotation)
                .dest(current_position_point),
        );
    }
    pub fn get_hitbox(&self) -> Hitbox {
        Hitbox {
            width: (self.size + STROKE_WIDTH) * 2.5,
            height: (self.size + STROKE_WIDTH) * 2.5,
            bottom_left: (
                self.position.0 as f32 - (self.size * 1.25),
                self.position.1 as f32 - (self.size * 1.25),
            ),
        }
    }
    pub fn draw_and_reach_base_check(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> bool {
        let mut current_rotation: f32 = 0.;
        let mut mesh_builder = MeshBuilder::new();
        let current_time = ctx.time.time_since_start().as_secs_f32();
        let time_dif = current_time - self.time_since_path_built;
        let path_index: usize = (time_dif * self.speed as f32) as usize;
        let half_size = self.size / 2.;
        let mut has_reached_base = false;
        //move position to the next parts of the path
        if path_index < self.path.len() {
            self.position = self.path.get(path_index).unwrap().to_owned();
        }
        if self.position.0 > -35
            && self.position.0 < 35
            && self.position.1 < 50
            && self.position.1 > -50
        {
            has_reached_base = true;
        }
        let orgin = (0.0 as f32, 0.0 as f32);
        // build the triangle around the new point
        mesh_builder
            .polygon(
                graphics::DrawMode::fill(),
                &[
                    [orgin.0 as f32, orgin.1 as f32 + self.size as f32],
                    [orgin.0 as f32 - half_size, orgin.1 as f32 - self.size],
                    [orgin.0 as f32 + half_size, orgin.1 as f32 - self.size],
                ],
                Color::RED,
            )
            .unwrap();
        let mesh_data = mesh_builder.build();
        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);

        //aim it towards the next direction and set the current roation to it
        if path_index < self.path.len() - 1 {
            let (next_x, next_y) = self.path.get(path_index + 1).unwrap().to_owned();
            //const rotation_bottom = 0.0;
            //const rotation_top_rotaiton = PI;
            //const rotation_top_left = (3. * PI) / 4.;
            //const rotation_top_right = (5. * PI) / 4.;
            //const rotation_bottom_left = PI /4.;
            //const rotation_bottom?_right = (PI * 7.) /4.;
            //const rotation_right = (3. * PI) / 2.;
            //const rotation_left = PI /2.;
            let corrected_x = self.position.0;
            let corrected_y = -self.position.1;
            let corrected_next_x = next_x;
            let corrected_next_y = -next_y;
            if corrected_x < corrected_next_x {
                if corrected_y < corrected_next_y {
                    // looking up right
                    current_rotation = ROTATION_TOP_RIGHT;
                } else if corrected_y > corrected_next_y {
                    // looking down right
                    current_rotation = ROTATION_Bottom_RIGHT;
                } else {
                    //looking right
                    current_rotation = ROTATION_RIGHT;
                }
            } else if corrected_x > corrected_next_x {
                // looking left
                if corrected_y < corrected_next_y {
                    // looking up left
                    current_rotation = ROTATION_TOP_LEFT;
                } else if corrected_y > corrected_next_y {
                    // looking down left
                    current_rotation = ROTATION_Bottom_LEFT;
                } else {
                    // looking left
                    current_rotation = ROTATION_LEFT;
                }
            } else {
                if corrected_y < corrected_next_y {
                    // looking up
                    current_rotation = ROTATION_TOP;
                } else {
                    // looking down
                    current_rotation = ROTATION_BOTTOM;
                }
            }
        }
        let current_position_point = Point2 {
            x: self.position.0 as f32,
            y: self.position.1 as f32,
        };
        self.last_rotation = current_rotation;
        canvas.draw(
            &mesh,
            DrawParam::default()
                .rotation(current_rotation)
                .dest(current_position_point),
        );
        has_reached_base
    }
}
