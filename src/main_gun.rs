use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect};
use ggez::input::keyboard::{self, KeyInput};
use ggez::mint::Point2;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use libm::sqrt;
use nalgebra::base::Vector2;
use nalgebra::geometry::Rotation2;
use nalgebra::{center, Rotation};
use std::default;
use std::ops::Add;
use std::time::Duration;

use crate::enemy;
#[derive(Default)]
struct TargetInfo {
    x: f32,
    y: f32,
    rotation: Rotation<f32, 2>,
    rotation_started: bool,
}
struct ExplosionInfo {
    x: f32,
    y: f32,
    started_time: f32,
    added_to_shake_meter: bool,
}
#[derive(Default)]
pub struct MainGun {
    pub(crate) enabled: bool,
    pub(crate) shell_explosive_radius: f32,
    pub(crate) damage: f32,
    pub(crate) explosion_info_list: Vec<ExplosionInfo>,
    pub(crate) fired_count: u32,
    pub(crate) last_fired: f32,
    pub(crate) since_fired: f32,
    pub(crate) shooting_duration: f32,
    pub(crate) current_rotation: f32,
    pub(crate) target_info_list: Vec<TargetInfo>,
    pub(crate) rotation_speed_per_second: f32,
    pub(crate) last_rotation: f32,
}
impl MainGun {
    fn get_barrel_segment_positions(&self) -> Vec<Rect> {
        let initial_animation_length = 0.15 * self.shooting_duration;
        let return_animation_length = 0.85 * self.shooting_duration;

        let base_radius = 9.5; //minused .5 for overlap

        let longest_barrel_height = 15.;
        let longest_barrel_dif = 9.;

        let middle_barrel_height = 6.;
        let middle_barrel_dif = 2.;

        let smallest_barrel_height = 3.;
        let smallest_barrel_dif = 1.;

        let mut barrels: Vec<Rect> = Vec::new();
        if self.since_fired < self.shooting_duration && self.fired_count > 0 {
            if self.since_fired < initial_animation_length {
                //first pulling back shot annimation
                let percentage_through_animation = self.since_fired / initial_animation_length;
                barrels.push(Rect {
                    x: -2.,
                    y: 0.0
                        - base_radius
                        - smallest_barrel_height
                        - middle_barrel_height
                        - longest_barrel_height
                        + (percentage_through_animation
                            * (longest_barrel_dif + middle_barrel_dif + smallest_barrel_dif)),
                    h: 2.,
                    w: 4.,
                });
                barrels.push(Rect {
                    x: -1.5,
                    y: 0.0
                        - base_radius
                        - smallest_barrel_height
                        - middle_barrel_height
                        - longest_barrel_height
                        + (percentage_through_animation
                            * (longest_barrel_dif + middle_barrel_dif + smallest_barrel_dif)),
                    h: longest_barrel_height - (percentage_through_animation * longest_barrel_dif),
                    w: 3.,
                });
                barrels.push(Rect {
                    x: -2.5,
                    y: 0.0 - base_radius - smallest_barrel_height - middle_barrel_height
                        + (percentage_through_animation
                            * (middle_barrel_dif + smallest_barrel_dif)),
                    h: middle_barrel_height - (percentage_through_animation * middle_barrel_dif),
                    w: 5.,
                });
                barrels.push(Rect {
                    x: -3.,
                    y: 0.0 - base_radius - smallest_barrel_height
                        + (percentage_through_animation * smallest_barrel_dif),
                    h: smallest_barrel_height
                        - (percentage_through_animation * smallest_barrel_dif),
                    w: 6.,
                });
            } else {
                //returning from shot animation
                let percentage_through_animation =
                    1. - ((self.since_fired - initial_animation_length) / return_animation_length);
                barrels.push(Rect {
                    x: -2.,
                    y: 0.0
                        - base_radius
                        - smallest_barrel_height
                        - middle_barrel_height
                        - longest_barrel_height
                        + (percentage_through_animation
                            * (longest_barrel_dif + middle_barrel_dif + smallest_barrel_dif)),
                    h: 2.,
                    w: 4.,
                });
                barrels.push(Rect {
                    x: -1.5,
                    y: 0.0
                        - base_radius
                        - smallest_barrel_height
                        - middle_barrel_height
                        - longest_barrel_height
                        + (percentage_through_animation
                            * (longest_barrel_dif + middle_barrel_dif + smallest_barrel_dif)),
                    h: longest_barrel_height - (percentage_through_animation * longest_barrel_dif),
                    w: 3.,
                });
                barrels.push(Rect {
                    x: -2.5,
                    y: 0.0 - base_radius - smallest_barrel_height - middle_barrel_height
                        + (percentage_through_animation
                            * (middle_barrel_dif + smallest_barrel_dif)),
                    h: middle_barrel_height - (percentage_through_animation * middle_barrel_dif),
                    w: 5.,
                });
                barrels.push(Rect {
                    x: -3.,
                    y: 0.0 - base_radius - smallest_barrel_height
                        + (percentage_through_animation * smallest_barrel_dif),
                    h: smallest_barrel_height
                        - (percentage_through_animation * smallest_barrel_dif),
                    w: 6.,
                });
            }
        } else {
            //println!("Gun is in idle");
            barrels.push(Rect {
                x: -2.,
                y: 0.0
                    - base_radius
                    - smallest_barrel_height
                    - middle_barrel_height
                    - longest_barrel_height,
                h: 2.,
                w: 4.,
            });
            barrels.push(Rect {
                x: -1.5,
                y: 0.0
                    - base_radius
                    - smallest_barrel_height
                    - middle_barrel_height
                    - longest_barrel_height,
                h: longest_barrel_height,
                w: 3.,
            });
            barrels.push(Rect {
                x: -2.5,
                y: 0.0 - base_radius - smallest_barrel_height - middle_barrel_height,
                h: middle_barrel_height,
                w: 5.,
            });
            barrels.push(Rect {
                x: -3.,
                y: 0.0 - base_radius - smallest_barrel_height,
                h: smallest_barrel_height,
                w: 6.,
            });
        }
        return barrels;
    }
    fn draw_explosions(&mut self, canvas: &mut Canvas, ctx: &mut Context, shake_meter: &mut u8) {
        if self.explosion_info_list.len() != 0 {
            if self.explosion_info_list.len() != 0 {
                let current_time = ctx.time.time_since_start().as_secs_f32();
                let mut current_explosion_index = self.explosion_info_list.len() - 1;
                let first_animation_length = 0.5;
                let second_animation_length = 4.;
                let third_animation_length = 2.;
                let mut deleted = false;
                let mut explosion = MeshBuilder::new();
                loop {
                    let cur_explosion = &mut self.explosion_info_list[current_explosion_index];
                    let time_since_explosion = current_time - cur_explosion.started_time;
                    if time_since_explosion < first_animation_length {
                        let percentage_through = time_since_explosion / first_animation_length;
                        //create and grow yellow circle
                        explosion
                            .circle(
                                graphics::DrawMode::fill(),
                                [cur_explosion.x, -cur_explosion.y],
                                self.shell_explosive_radius * percentage_through,
                                0.1,
                                Color::YELLOW,
                            )
                            .unwrap();
                    } else if time_since_explosion
                        < first_animation_length + second_animation_length
                    {
                        if !cur_explosion.added_to_shake_meter {
                            if shake_meter.add(20) < 100 as u8 {
                                *shake_meter = shake_meter.add(20);
                            } else if *shake_meter < 100 as u8 {
                                *shake_meter = 100 as u8;
                            }
                            cur_explosion.added_to_shake_meter = true;
                        }
                        let percentage_through = (time_since_explosion - first_animation_length)
                            / second_animation_length;
                        //create red and shrink yellow
                        explosion
                            .circle(
                                graphics::DrawMode::fill(),
                                [cur_explosion.x, -cur_explosion.y],
                                self.shell_explosive_radius,
                                0.1,
                                Color::from_rgb(
                                    220 - (115. * percentage_through) as u8,
                                    20 + (85. * percentage_through) as u8,
                                    60 + (45. * percentage_through) as u8,
                                ),
                            )
                            .unwrap()
                            .circle(
                                graphics::DrawMode::fill(),
                                [cur_explosion.x, -cur_explosion.y],
                                self.shell_explosive_radius * (1. - percentage_through),
                                0.1,
                                Color::YELLOW,
                            )
                            .unwrap();
                    } else if time_since_explosion
                        < (first_animation_length
                            + second_animation_length
                            + third_animation_length)
                    {
                        let percentage_through = (time_since_explosion
                            - first_animation_length
                            - second_animation_length)
                            / third_animation_length;
                        //grow grey circle over while still shrinking
                        explosion
                            .circle(
                                graphics::DrawMode::fill(),
                                [cur_explosion.x, -cur_explosion.y],
                                self.shell_explosive_radius,
                                0.1,
                                Color::from_rgba(
                                    220 - (115. * percentage_through) as u8,
                                    20 + (85. * percentage_through) as u8,
                                    60 + (45. * percentage_through) as u8,
                                    (255. * (1. - percentage_through)) as u8,
                                ),
                            )
                            .unwrap();
                    } else {
                        self.explosion_info_list.remove(current_explosion_index);
                        deleted = true;
                    }
                    if !deleted {
                        let mesh_data = explosion.build();
                        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);
                        canvas.draw(&mesh, DrawParam::default());
                    }
                    if current_explosion_index == 0 {
                        break;
                    }
                    current_explosion_index -= 1;
                }
            }
        }
    }
    pub fn move_and_check_fire(
        &mut self,
        time_since_start: Duration,
        enemy_alive_list: &mut Vec<enemy::Enemy>,
    ) {
        let time_since_start_sec = time_since_start.as_secs_f32();
        if self.target_info_list.len() == 0 || self.since_fired < self.shooting_duration {
            self.last_rotation = time_since_start_sec;
            return;
        }
        //radian range is 0 - 2pi
        //first we get left and right distances
        let start_rotation = Rotation2::new(-self.current_rotation);

        let cur_rotation = start_rotation;
        // on first starting the rotation
        if !self.target_info_list[0].rotation_started {
            //set the time it started
            self.target_info_list[0].rotation_started = true;
            //set the rotation
            let target_vec2 = Vector2::new(self.target_info_list[0].x, self.target_info_list[0].y);
            let turret_pos = Vector2::new(0.0f32, 0.0f32);
            let turret_axis = Vector2::new(0., 1.);
            self.target_info_list[0].rotation =
                Rotation2::rotation_between(&turret_axis, &(target_vec2 - turret_pos));
        }
        let needed_rotation = cur_rotation
            .rotation_to(&self.target_info_list[0].rotation)
            .angle();

        //find ammount to rotate
        let time_diff_seconds = time_since_start_sec - self.last_rotation;
        let movement_ammount = time_diff_seconds * self.rotation_speed_per_second;

        // choose a way to turn and do it
        if needed_rotation < 0.0 {
            if needed_rotation * -1. < movement_ammount {
                self.current_rotation = -self.target_info_list[0].rotation.angle();
                self.fire(time_since_start, enemy_alive_list);
            } else {
                self.current_rotation += movement_ammount;
            }
        } else {
            if needed_rotation < movement_ammount {
                self.current_rotation = -self.target_info_list[0].rotation.angle();
                self.fire(time_since_start, enemy_alive_list);
            } else {
                self.current_rotation -= movement_ammount;
            }
        }
        self.last_rotation = time_since_start_sec;
    }
    pub fn draw(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        enemy_alive_list: &mut Vec<enemy::Enemy>,
        shake_meter: &mut u8,
    ) {
        self.draw_explosions(canvas, ctx, shake_meter);
        let time_since_start_as_sec = ctx.time.time_since_start().as_secs_f32();
        self.since_fired = time_since_start_as_sec - self.last_fired;
        let mut mesh_builder = MeshBuilder::new();
        let barrel_positions = self.get_barrel_segment_positions();
        let mut triangle_opacity = 0;
        if self.since_fired < 0.05 {
            triangle_opacity = 255
        }
        mesh_builder
            .circle(
                graphics::DrawMode::fill(),
                [0., 0.],
                10.,
                0.0001,
                Color::from_rgb(60, 60, 60),
            )
            .unwrap()
            .rectangle(
                graphics::DrawMode::fill(),
                barrel_positions[0],
                Color::from_rgb(60, 60, 60),
            )
            .unwrap()
            .rectangle(
                graphics::DrawMode::fill(),
                barrel_positions[1],
                Color::from_rgb(60, 60, 60),
            )
            .unwrap()
            .rectangle(
                graphics::DrawMode::fill(),
                barrel_positions[2],
                Color::from_rgb(60, 60, 60),
            )
            .unwrap()
            .rectangle(
                graphics::DrawMode::fill(),
                barrel_positions[3],
                Color::from_rgb(60, 60, 60),
            )
            .unwrap()
            .polygon(
                graphics::DrawMode::fill(),
                &[[-2., -32.5], [-2., -30.5], [-8., -31.5]],
                Color::from_rgba(255, 255, 51, triangle_opacity),
            )
            .unwrap()
            .polygon(
                graphics::DrawMode::fill(),
                &[[2., -32.5], [2., -30.5], [8., -31.5]],
                Color::from_rgba(255, 255, 51, triangle_opacity),
            )
            .unwrap();
        // MAIN GOAL: figure out the positions of each of the barrel segments and draw them
        let mesh_data = mesh_builder.build();
        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);
        self.move_and_check_fire(ctx.time.time_since_start(), enemy_alive_list);
        canvas.draw(&mesh, DrawParam::default().rotation(self.current_rotation))
    }
    fn draw_artillary_round(
        &self,
        left_x: f32,
        bottom_y: f32,
        scale: f32,
        ctx: &mut Context,
        canvas: &mut Canvas,
    ) {
        //builds an artillary round based on the bottom left corner a size ratio
        let bottom_y = -bottom_y;
        let width = 1. * scale;
        let mut artillary_round = MeshBuilder::new();
        let height_of_casing = -5. * scale;
        let height_of_neck = -3. * scale;
        let height_of_tip = -2. * scale;
        let neck_loss = 0.2 * scale;
        artillary_round
            .rectangle(
                DrawMode::fill(),
                Rect {
                    x: left_x,
                    y: bottom_y,
                    w: width,
                    h: height_of_casing,
                },
                Color::from_rgb(69, 75, 27),
            )
            .unwrap()
            .polygon(
                DrawMode::fill(),
                &[
                    [left_x, bottom_y + height_of_casing],
                    [
                        left_x + neck_loss,
                        bottom_y + height_of_casing + height_of_neck,
                    ],
                    [
                        left_x + (width - neck_loss),
                        bottom_y + height_of_casing + height_of_neck,
                    ],
                    [left_x + width, bottom_y + height_of_casing],
                ],
                Color::from_rgb(69, 75, 27),
            )
            .unwrap()
            .polygon(
                DrawMode::fill(),
                &[
                    [
                        left_x + neck_loss,
                        bottom_y + height_of_casing + height_of_neck,
                    ],
                    [
                        left_x + (width / 2.),
                        bottom_y + height_of_casing + height_of_neck + height_of_tip,
                    ],
                    [
                        left_x + (width - neck_loss),
                        bottom_y + height_of_casing + height_of_neck,
                    ],
                ],
                Color::YELLOW,
            )
            .unwrap();
        let mesh_data = artillary_round.build();
        let mesh = Mesh::from_data(&ctx.gfx, mesh_data);
        canvas.draw(&mesh, DrawParam::default())
    }
    pub fn draw_ammo_loader(&self, ctx: &mut Context, canvas: &mut Canvas) {
        //let mut ammo_loader = MeshBuilder::new();
        let mut percentage_through = 0.;
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::from_rgb(128, 128, 128))
                .scale([24., 11.5])
                .dest([0., -5.5]),
        );
        if self.shooting_duration > self.since_fired {
            percentage_through = self.since_fired / self.shooting_duration;
            //this is the 8th shot comming up
            self.draw_artillary_round(
                22. + (0.75 + (0.25 * percentage_through)),
                -2.5 - (2.5 * percentage_through),
                0.5 + (percentage_through / 2.),
                ctx,
                canvas,
            );
        }
        let mut counter = 0.;
        while counter < 16. {
            if counter % 2. != 1. {
                counter += 1.
            }
            self.draw_artillary_round(
                8. + counter - (percentage_through * 2.),
                -5.,
                1.,
                ctx,
                canvas,
            );
            counter += 1.;
        }
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::BLACK)
                .scale([3., 12.])
                .dest([23.5, -6.]),
        );
    }
    pub fn initiate_fire_sequence(&mut self, x: f32, y: f32) {
        //feeding a target to the maingun
        if self.enabled {
            self.target_info_list.push(TargetInfo {
                x: x,
                y: y,
                rotation: Rotation::default(),
                rotation_started: false,
            });
        }
        println!("added new target at x:{} y:{}", x, y);
    }
    pub fn fire(&mut self, time_since_start: Duration, enemy_alive_list: &mut Vec<enemy::Enemy>) {
        if (self.since_fired > self.shooting_duration || self.fired_count == 0) && self.enabled {
            let current_time = time_since_start.as_secs_f32();
            println!("Gun Has Fired");
            self.fired_count = self.fired_count + 1;
            self.last_fired = current_time;
            //build the explosion
            let center_of_explosion = self.target_info_list.get(0).unwrap();
            self.explosion_info_list.push(ExplosionInfo {
                x: center_of_explosion.x,
                y: center_of_explosion.y,
                started_time: current_time,
                added_to_shake_meter: false,
            });
            //collision check
            //first check if a box would have hit
            let hitbox_top_left = (
                center_of_explosion.x - self.shell_explosive_radius,
                center_of_explosion.y + self.shell_explosive_radius,
            );
            let hitbox_size = self.shell_explosive_radius * 2.;
            for enemy in enemy_alive_list {
                //correct y to be negative
                let enemy_x = enemy.position.0 as f32;
                let enemy_y = -enemy.position.1 as f32;
                // check if its in the box version of the explosiown(less expensive to check?)
                println!(
                    "check if enemy at ({},{}) is in corner ({},{}), with size: {})",
                    enemy_x, enemy_y, hitbox_top_left.0, hitbox_top_left.1, hitbox_size
                );
                if enemy_x > hitbox_top_left.0
                    && enemy_x < hitbox_top_left.0 + hitbox_size
                    && enemy_y < hitbox_top_left.1
                    && enemy_y < hitbox_top_left.1 + hitbox_size
                {
                    println!("Enemy in  Box HITBOX");
                    // check the circle hitbox wit distance formula from the center of the circle
                    let dif_x = enemy_x - center_of_explosion.x;
                    let dif_y = enemy_y - center_of_explosion.y;
                    let distance_from_center: f32 =
                        sqrt(dif_x.powf(2.) as f64 + dif_y.powf(2.) as f64) as f32;
                    if distance_from_center < self.shell_explosive_radius {
                        //the target was in the circle
                        enemy.health -= self.damage;
                        println!("Enemy was hit");
                    }
                }
            }
            self.target_info_list.remove(0);
        }
    }
}
