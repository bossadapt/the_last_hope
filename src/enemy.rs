use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh, MeshBuilder, Rect, Transform};
use ggez::mint::Point2;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use num::abs;
use pathfinding::grid;
use std::f32::consts::PI;

use crate::{Direction, Map};

const ORGIN: (f32, f32) = (0.0 as f32, 0.0 as f32);
///ROTATION BASED ON TOP LEFT to BOTTOM RIGHT
const ROTATION: [f32;4] =[PI,(3. * PI) / 2.,PI / 2., 0.0];
//const ROTATION_BOTTOM: f32 = 0.0;
// const ROTATION_TOP: f32 = PI;
// const ROTATION_TOP_LEFT: f32 = (3. * PI) / 4.;
// const ROTATION_TOP_RIGHT: f32 = (5. * PI) / 4.;
// const ROTATION_Bottom_LEFT: f32 = PI / 4.;
//const ROTATION_Bottom_RIGHT: f32 = (PI * 7.) / 4.;
// const ROTATION_RIGHT: f32 = (3. * PI) / 2.;
// const ROTATION_LEFT: f32 = PI / 2.;
const STROKE_WIDTH: f32 = 2.0;
const DIRECTIONS_REVERSED:[(f32,f32);4] = [(0.,-1.),(1.,0.),(-1.,0.),(0.,1.)];
#[derive(Clone)]
pub struct Enemy {
    pub(crate) health: f32,
    pub(crate) position: (f32, f32),
    pub(crate) size: f32,
    pub(crate) rotation: f32,
    pub(crate) speed: u32,
    pub(crate) building_hit: Option<u32>
}
pub struct Hitbox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) bottom_left: (f32, f32),
}
impl Enemy {
    pub fn draw_dead(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let mut mesh_builder = MeshBuilder::new();
        let half_size = self.size / 2.;
        mesh_builder
            .polygon(
                graphics::DrawMode::stroke(STROKE_WIDTH),
                &[
                    [ORGIN.0 as f32, ORGIN.1 as f32 + self.size as f32],
                    [ORGIN.0 as f32 - half_size, ORGIN.1 as f32 - self.size],
                    [ORGIN.0 as f32 + half_size, ORGIN.1 as f32 - self.size],
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
                .rotation(self.rotation)
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
    pub fn draw_and_reach_base_check(&mut self, ctx: &mut Context, canvas: &mut Canvas, map: &Map) -> bool {
        let mut current_rotation: f32 = 0.;
        let mut mesh_builder = MeshBuilder::new();
        let time_dif: f32 = ctx.time.delta().as_secs_f32();
        let half_size = self.size / 2.;
        //TODO: make sure this does not only cause damage to the main building, and affects the one where the grid is
        let mut has_reached_objective = false;
        let grid_position = Map::convert_position_to_grid_position(self.position);
        let current_gridspace = &map.map[grid_position.0][grid_position.1];
        if current_gridspace.building.is_some()
        {
            has_reached_objective = true;
        }
        // UPDATING POSITION 
        if let Some(direction) = current_gridspace.direction.clone(){
            println!("direction found: {}",direction.clone() as usize);
            println!("speed: {}",self.speed);
            println!("timedif: {}",time_dif);
            println!("ammount of movement {}",(DIRECTIONS_REVERSED[direction.clone() as usize].0 as f32 * self.speed as f32 * time_dif) as  f32);
            self.position.0 = self.position.0 + (DIRECTIONS_REVERSED[direction.clone() as usize].0 as f32 * self.speed as f32 * time_dif);
            self.position.1 = self.position.1 + (DIRECTIONS_REVERSED[direction as usize].1 as f32 * self.speed as f32 * time_dif);
        }else{
            println!("direction does not exist");
        }
        // build the triangle around the new point
        mesh_builder
            .polygon(
                graphics::DrawMode::fill(),
                &[
                    [ORGIN.0 as f32, ORGIN.1 as f32 + self.size as f32],
                    [ORGIN.0 as f32 - half_size, ORGIN.1 as f32 - self.size],
                    [ORGIN.0 as f32 + half_size, ORGIN.1 as f32 - self.size],
                ],
                Color::RED,
            )
            .unwrap();
        let mesh_data = mesh_builder.build();
        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);
        // start
        //UPDATING ROTATION
        if let Some(direction) = current_gridspace.direction.clone(){
            //const rotation_bottom = 0.0;
            //const rotation_top_rotaiton = PI;
            //const rotation_top_left = (3. * PI) / 4.;
            //const rotation_top_right = (5. * PI) / 4.;
            //const rotation_bottom_left = PI /4.;
            //const rotation_bottom?_right = (PI * 7.) /4.;
            //const rotation_right = (3. * PI) / 2.;
            //const rotation_left = PI /2.;
            current_rotation = ROTATION[direction as usize];
            // BASED ON OLD ASTAR ARRAY
            //let corrected_x = self.position.0;
            //let corrected_y = -self.position.1;
            //let corrected_next_x = next_x;
            //let corrected_next_y = -next_y;
            // if corrected_x < corrected_next_x {
            //     if corrected_y < corrected_next_y {
            //         // looking up right
            //         current_rotation = ROTATION_TOP_RIGHT;
            //     } else if corrected_y > corrected_next_y {
            //         // looking down right
            //         current_rotation = ROTATION_Bottom_RIGHT;
            //     } else {
            //         //looking right
            //         current_rotation = ROTATION_RIGHT;
            //     }
            // } else if corrected_x > corrected_next_x {
            //     // looking left
            //     if corrected_y < corrected_next_y {
            //         // looking up left
            //         current_rotation = ROTATION_TOP_LEFT;
            //     } else if corrected_y > corrected_next_y {
            //         // looking down left
            //         current_rotation = ROTATION_Bottom_LEFT;
            //     } else {
            //         // looking left
            //         current_rotation = ROTATION_LEFT;
            //     }
            // } else {
            //     if corrected_y < corrected_next_y {
            //         // looking up
            //         current_rotation = ROTATION_TOP;
            //     } else {
            //         // looking down
            //         current_rotation = ROTATION_BOTTOM;
            //     }
            // }
        }else if let Some(building_grid_info) = current_gridspace.building.clone(){
            self.building_hit = Some(building_grid_info.id);
            return true;
        }
        let current_position_point = Point2 {
            x: self.position.0 as f32,
            y: self.position.1 as f32,
        };
        self.rotation = current_rotation;
        canvas.draw(
            &mesh,
            DrawParam::default()
                .rotation(current_rotation)
                .dest(current_position_point),
        );
        has_reached_objective
    }
}
