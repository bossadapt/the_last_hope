use enemy::Enemy;
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{
    self, Canvas, Color, DrawParam, Drawable, Mesh, MeshBuilder, PxScale, Rect, Text, TextFragment,
};
use ggez::input::keyboard::{self, KeyInput};
use ggez::mint::Point2;
use ggez::winit::dpi::Position;
use ggez::winit::event::VirtualKeyCode;
use ggez::{conf, Context, ContextBuilder, GameError, GameResult};
use libm::atan2f;
use nalgebra::base::Vector2;
use nalgebra::geometry::Rotation2;
use nalgebra::Rotation;
use num::abs;
use pathfinding::prelude::astar;
use rand::Rng;
use std::default;
use std::time::Duration;
use worker::Task;
const DEFAULT_CAM_SIZE: f32 = 100.0;
use std::f32::consts::PI;
mod enemy;
mod main_gun;
mod worker;
fn main() {
    // Make a Context.
    let mut cf = conf::Conf::new();
    cf.window_setup.title = "The Last Hope".to_owned();
    let (mut ctx, event_loop) = ContextBuilder::new("The Last Hope", "Bossadapt")
        .default_conf(cf)
        .build()
        .unwrap();
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut ctx);
    // Run!
    event::run(ctx, event_loop, my_game);
}
//before
fn radian_from_points(orgin_location: &[f32; 2], target_location: &[f32; 2]) -> f32 {
    let x_dif = orgin_location[0] - target_location[0];
    let y_dif = orgin_location[1] - target_location[1];
    let radian = atan2f(-y_dif, -x_dif);
    return radian;
}
#[derive(Clone)]
struct Map {
    sentries: Vec<(i32, i32)>,
    barriers: Vec<(i32, i32)>,
}
impl Map {
    pub fn find_moveable_options(self, x: i32, y: i32) -> Vec<(i32, i32)> {
        let mut movement_options: Vec<(i32, i32)> = vec![
            (x + 1, y + 1),
            (x + 1, y),
            (x + 1, y - 1),
            (x, y + 1),
            (x, y - 1),
            (x - 1, y + 1),
            (x - 1, y),
            (x - 1, y - 1),
        ];
        let mut index = movement_options.len() - 1;
        //this the number it gets assigned when it runs out of bounds
        while true {
            let current_cord = movement_options.get(index).unwrap();
            if current_cord.0 > 500
                || current_cord.1 > 500
                || current_cord.0 < -500
                || current_cord.1 < -500
            {
                movement_options.remove(index);
            } else if self.sentries.contains(current_cord) || self.barriers.contains(current_cord) {
                movement_options.remove(index);
            }
            if index == 0 {
                break;
            }
            index -= 1;
        }
        movement_options
    }
}
struct MyGame {
    main_gun: main_gun::MainGun,
    map: Map,
    player_max_health: f32,
    player_current_health: f32,
    enemy_alive_list: Vec<enemy::Enemy>,
    enemy_dead_list: Vec<enemy::Enemy>,
    worker_task_list: Vec<worker::Task>,
    worker_list: Vec<worker::Worker>,
    rooftop_view: bool,
    camera_zoom_ratio: f32,
    camera_x: f32,
    camera_y: f32,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        MyGame {
            player_max_health: 1000.,
            player_current_health: 1000.,
            worker_list: Vec::new(),
            worker_task_list: Vec::new(),
            main_gun: main_gun::MainGun {
                shooting_duration: 2.,
                enabled: true,
                rotation_speed_per_second: PI / 10.0,
                shell_explosive_radius: 50.,
                damage: 100.,
                ..Default::default()
            },
            map: Map {
                sentries: Vec::new(),
                barriers: Vec::new(),
            },
            enemy_alive_list: Vec::new(),
            enemy_dead_list: Vec::new(),
            rooftop_view: true,
            camera_zoom_ratio: 1.,
            camera_x: -50.,
            camera_y: -50.,
        }
    }
    fn spawn_enemy(&mut self, ctx: &Context) -> Result<(), GameError> {
        let random_ratio: f32 = rand::thread_rng().gen_range(0.0..1.);
        let base_health: f32 = 100. * (random_ratio + 0.5);
        let base_size = 20. * (random_ratio + 0.5);
        let random_side: i8 = rand::thread_rng().gen_range(1..5);
        let current_time = ctx.time.time_since_start().as_secs_f32();
        let mut random_side_length = (500. * random_ratio) as i32;
        if rand::random() {
            random_side_length = random_side_length * -1;
        }
        println!("current side: {}", random_side);
        let position_generated: (i32, i32) = match random_side {
            1 => (-500, random_side_length),
            2 => (500, random_side_length),
            3 => (random_side_length, -500),
            4 => (random_side_length, 500),
            _ => {
                panic!()
            }
        };
        println!(
            "Enemy Spawned at {},{} with size {} with alread {} enemies",
            &position_generated.0,
            &position_generated.1,
            &base_size,
            self.enemy_alive_list.len()
        );
        let mut new_enemy = Enemy {
            health: base_health,
            size: base_size,
            position: position_generated,
            path: Vec::new(),
            speed: 15,
            last_rotation: 0.0,
            time_since_path_built: current_time,
        };
        println!("Path building started");
        new_enemy.path = self.build_path(position_generated, (0, 0)).unwrap().0;
        println!("Path building finished");
        self.enemy_alive_list.push(new_enemy);
        Ok(())
    }
    fn build_path(
        &self,
        start_location: (i32, i32),
        goal: (i32, i32),
    ) -> Option<(Vec<(i32, i32)>, u32)> {
        let result: Option<(Vec<(i32, i32)>, u32)> = astar(
            &start_location,
            |&(x, y)| {
                self.map
                    .clone()
                    .find_moveable_options(x, y)
                    .into_iter()
                    .map(|p| (p, 1))
            },
            |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
            |&p| p == goal,
        );
        result
    }
    fn offset_to_screen_cord(&self, ctx: &Context, screen_cord_wanted: &[f32; 2]) -> [f32; 2] {
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;
        let camera_world_view_width = DEFAULT_CAM_SIZE * self.camera_zoom_ratio * aspect_ratio;
        let camera_world_view_height = DEFAULT_CAM_SIZE * self.camera_zoom_ratio;
        let cord_wanted_x = screen_cord_wanted[0] * camera_world_view_width;
        let cord_wanted_y = screen_cord_wanted[1] * camera_world_view_height;
        let output_x = cord_wanted_x + (self.camera_x * aspect_ratio);
        let output_y = cord_wanted_y + self.camera_y;
        [output_x, output_y]
    }
    fn screen_cord_to_world_cord(&self, ctx: &Context, screen_cord: &[f32; 2]) -> [f32; 2] {
        // scale our x and y from [0, screen_width] to [0, 1]
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;
        let ndc = [
            screen_cord[0] / window_size.width as f32,
            screen_cord[1] / window_size.height as f32,
        ];

        // convert our NDC into world space
        let camera_world_view_width = DEFAULT_CAM_SIZE * self.camera_zoom_ratio;
        let camera_world_view_height = DEFAULT_CAM_SIZE * self.camera_zoom_ratio;
        let world_coord = [
            ndc[0] * camera_world_view_width * aspect_ratio + (self.camera_x * aspect_ratio),
            (ndc[1] * camera_world_view_height + self.camera_y) * -1.,
        ];

        println!("Converted {:?} to {:?}", screen_cord, world_coord);
        world_coord
    }
    fn switch_setting(&mut self) -> Result<(), GameError> {
        self.rooftop_view = !self.rooftop_view;
        self.main_gun.enabled = self.rooftop_view;
        Ok(())
    }
    fn change_camera_zoom(&mut self, zoom_increase: bool) -> Result<(), GameError> {
        let diffrence_in_cam_size = DEFAULT_CAM_SIZE * 0.1;
        if zoom_increase {
            //increase zoom
            let new_camera_zoom_ratio = self.camera_zoom_ratio - 0.1;
            if new_camera_zoom_ratio > 0.5 {
                self.camera_zoom_ratio = new_camera_zoom_ratio;
                self.camera_x += diffrence_in_cam_size / 2.;
                self.camera_y += diffrence_in_cam_size / 2.;
            }
        } else {
            //decrease zoom
            self.camera_zoom_ratio += 0.1;
            self.camera_x -= diffrence_in_cam_size / 2.;
            self.camera_y -= diffrence_in_cam_size / 2.;
        }
        println!("Current Zoom:{}", self.camera_zoom_ratio);
        Ok(())
    }
    fn change_camera_location(&mut self, key_pressed: char) -> Result<(), GameError> {
        match key_pressed {
            'w' => self.camera_y -= 2. * self.camera_zoom_ratio,
            'a' => self.camera_x -= 2. * self.camera_zoom_ratio,
            's' => self.camera_y += 2. * self.camera_zoom_ratio,
            'd' => self.camera_x += 2. * self.camera_zoom_ratio,
            _ => (),
        }
        Ok(())
    }
    fn manage_enemies(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        if self.enemy_alive_list.len() != 0 {
            let mut current_enemy_index: usize = self.enemy_alive_list.len() - 1;
            //check enemies for abnomalities and spawn
            while true {
                let current_enemy = &mut self.enemy_alive_list[current_enemy_index];
                if current_enemy.health < 0. {
                    //put the dead enemies in the deadlist
                    self.enemy_dead_list.push(
                        self.enemy_alive_list
                            .get(current_enemy_index)
                            .unwrap()
                            .clone(),
                    );
                    self.enemy_alive_list.remove(current_enemy_index);
                } else if current_enemy.draw_and_reach_base_check(ctx, canvas) {
                    //despawn the ones that reached the base and apply dmg
                    self.player_current_health -= self
                        .enemy_alive_list
                        .get(current_enemy_index)
                        .unwrap()
                        .health;
                    self.enemy_alive_list.remove(current_enemy_index);
                }
                if current_enemy_index == 0 {
                    break;
                }
                current_enemy_index -= 1;
            }
        }
        //draw dead enemies
        for enemy in &mut self.enemy_dead_list {
            enemy.draw_dead(ctx, canvas);
        }
    }
    fn draw_ui(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let uniform_og_scale = 30.0 * self.camera_zoom_ratio;
        let uniform_px_scale = PxScale::from(uniform_og_scale);
        let uniform_rescale = 0.1;
        let height_of_sections = uniform_og_scale * uniform_rescale;
        //build health text
        let health_text_format = format!(
            "{} / {}",
            self.player_current_health as i32, self.player_max_health as i32
        );
        let mut health_text_fragment = TextFragment::new(health_text_format);
        health_text_fragment.color = Some(Color::WHITE);
        health_text_fragment.scale = Some(uniform_px_scale);
        let health_text = Text::new(health_text_fragment);
        //let health_text_offset = self.offset_to_screen_cord(ctx, &[0.05, 0.03]);
        let health_screen_offset = self.offset_to_screen_cord(ctx, &[0.01, 0.01]);
        //build health bar
        let health_bar_meshbuilder = MeshBuilder::new();
        let health_bar_border_size = 1.5 * self.camera_zoom_ratio;
        let health_size: [f32; 2] = [30. * self.camera_zoom_ratio, 7. * self.camera_zoom_ratio];
        let percent_health = self.player_current_health / self.player_max_health;
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::BLACK)
                .scale([health_size[0], health_size[1]])
                .dest([health_screen_offset[0], health_screen_offset[1]]),
        );
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::RED)
                .scale([
                    (health_size[0] - health_bar_border_size) * percent_health,
                    health_size[1] - health_bar_border_size,
                ])
                .dest([
                    health_screen_offset[0] + health_bar_border_size / 2.,
                    health_screen_offset[1] + health_bar_border_size / 2.,
                ]),
        );
        let health_text_measure = health_text.measure(ctx).unwrap();
        let health_text_offset_x: f32 =
            (health_size[0] / 2.) - ((health_text_measure.x * uniform_rescale) / 2.);
        let health_text_offset_y: f32 =
            (health_size[1] / 2.) - ((health_text_measure.y * uniform_rescale) / 2.);
        health_text.draw(
            canvas,
            DrawParam::default()
                .scale([uniform_rescale, uniform_rescale])
                .dest([
                    health_screen_offset[0] + health_text_offset_x,
                    health_screen_offset[1] + health_text_offset_y,
                ]),
        )
    }
    fn manage_workers(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        if self.worker_task_list.len() > 0 {
            let mut current_worker_index: usize = self.worker_list.len() - 1;

            while true {
                let current_worker = &mut self.worker_list[current_worker_index];
                if current_worker.health < 0. {
                    self.worker_list.remove(current_worker_index);
                } else {
                    if current_worker.avalible_for_task {
                        if self.worker_task_list.len() != 0 {
                            current_worker.task = self.worker_task_list[0].clone();
                            current_worker.time_since_path_started =
                                ctx.time.time_since_start().as_secs_f32();
                            let start_location: (i32, i32) = current_worker.position;
                            let goal_location: (i32, i32) = self.worker_task_list[0].goals[0];
                            current_worker.avalible_for_task = false;
                            let path = self.build_path(start_location, goal_location).unwrap().0;
                            let current_worker = &mut self.worker_list[current_worker_index];
                            current_worker.path = path;
                            self.worker_task_list.remove(0);
                        }
                    } else if current_worker.ready_for_new_path {
                    }
                    let current_worker = &mut self.worker_list[current_worker_index];
                    current_worker.draw(ctx, canvas);
                }
                if current_worker_index == 0 {
                    break;
                }
                current_worker_index -= 1;
            }
        }
    }
    fn initiate_task(&mut self, mouse_x: f32, mouse_y: f32) {
        let mouse_y = -mouse_y;
        if !self.rooftop_view {
            for bad_guy in &self.enemy_dead_list {
                let current_hitbox = bad_guy.get_hitbox();
                if mouse_x > current_hitbox.bottom_left.0
                    && mouse_x < current_hitbox.bottom_left.0 + current_hitbox.width
                    && mouse_y > current_hitbox.bottom_left.1
                    && mouse_y < current_hitbox.bottom_left.1 + current_hitbox.height
                {
                    // a dead enemy was clicked
                    let time_to_collect_body: f32 = 0.5;
                    let time_to_deposit_body: f32 = 0.5;
                    let home_cord: (i32, i32) = (0, 0);
                    let collect_dead_task = worker::Task {
                        task_times: vec![time_to_collect_body, time_to_deposit_body],
                        goals: vec![bad_guy.position, home_cord],
                    };
                    self.worker_task_list.push(collect_dead_task);
                    break;
                }
            }
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Update code here...
        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        match _button {
            MouseButton::Left => {
                // creates new Circle and push to vector
                let [x, y] = self.screen_cord_to_world_cord(ctx, &[x, y]);
                self.main_gun.initiate_fire_sequence(x, y);
            }
            MouseButton::Right => {
                // creates new Circle and push to vector
                let [x, y] = self.screen_cord_to_world_cord(ctx, &[x, y]);
                self.initiate_task(x, y);
            }
            _ => {}
        }
        Ok(())
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        match input.keycode {
            Some(VirtualKeyCode::Tab) => self.switch_setting(),
            Some(VirtualKeyCode::Q) => self.change_camera_zoom(false),
            Some(VirtualKeyCode::E) => self.change_camera_zoom(true),
            Some(VirtualKeyCode::W) => self.change_camera_location('w'),
            Some(VirtualKeyCode::A) => self.change_camera_location('a'),
            Some(VirtualKeyCode::S) => self.change_camera_location('s'),
            Some(VirtualKeyCode::D) => self.change_camera_location('d'),
            Some(VirtualKeyCode::Z) => self.spawn_enemy(ctx),
            _ => Ok(()),
        }
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;
        canvas.set_screen_coordinates(Rect::new(
            self.camera_x * aspect_ratio,
            self.camera_y,
            (DEFAULT_CAM_SIZE * aspect_ratio) * self.camera_zoom_ratio,
            DEFAULT_CAM_SIZE * self.camera_zoom_ratio,
        ));
        if self.rooftop_view {
            // draw rooftop scene
            // default x: -50 y : -50
            //the floor
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::from_rgb(125, 125, 125))
                    .scale([74., 54.])
                    .dest([-37., -27.]),
            );
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::from_rgb(105, 105, 105))
                    .scale([70., 50.])
                    .dest([-35., -25.]),
            );
            self.main_gun.draw_ammo_loader(ctx, &mut canvas);
            self.main_gun
                .draw(ctx, &mut canvas, &mut self.enemy_alive_list);
        } else {
            // draw ground scene
            //the floor
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::from_rgb(128, 128, 128))
                    .scale([70., 50.])
                    .dest([-35., -25.]),
            );
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::BLACK)
                    .scale([10., 10.])
                    .dest([-5., -5.]),
            );
        }
        self.manage_workers(ctx, &mut canvas);
        self.manage_enemies(ctx, &mut canvas);
        self.draw_ui(ctx, &mut canvas);
        canvas.finish(ctx)
    }
}
