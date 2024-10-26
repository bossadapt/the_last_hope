use enemy::Enemy;
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{
    self, Canvas, Color, DrawParam, Drawable, Mesh, MeshBuilder, PxScale, Rect, Text, TextFragment,
};
use ggez::input::keyboard::{self, KeyInput};
use ggez::winit::event::VirtualKeyCode;
use ggez::{conf, Context, ContextBuilder, GameError, GameResult};
use libm::atan2f;
use num::cast::AsPrimitive;
use pathfinding::matrix::directions;
use pathfinding::prelude::astar;
use rand::Rng;
const DEFAULT_CAM_SIZE: f32 = 100.0;
use std::any::Any;
use std::default;
use std::f32::consts::PI;
use std::ops::{Div, Mul};
mod enemy;
mod main_gun;
mod worker;
use std::collections::{HashMap, VecDeque};
const DIRECTIONS:[(i32,i32);4] = [(0,1),(-1,0),(1,0),(0,-1)];
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
#[derive(Clone,Copy)]
enum BuildingType{
    Sentry,
    Baricade,
}
#[derive(Clone)]
enum Direction{
    TOP,
    LEFT,
    RIGHT,
    BOTTOM
}
impl Direction {
    fn new(index:usize) -> Self{
        match index{
            0=> Direction::TOP,
            1=> Direction::LEFT,
            2=> Direction::RIGHT,
            3=> Direction::BOTTOM,
            _=> panic!("Used an index outside of possible Directions")
        }
    }
}
#[derive(Clone)]
struct BuildingGridInfo{
    id: u32,
    typ: BuildingType,
}
///Will either be used to direct enemies or assign damage if an enemy attacks it
#[derive(Clone)]
struct GridSpace{
    building: Option<BuildingGridInfo>,
    direction: Option<Direction>
}
#[derive(Clone)]
struct Map {
    map: Vec<Vec<GridSpace>>,
}
///Grid system for placing objects and pathing enemies
impl Map {
    fn new(building_hash_map: &HashMap<u32,Building>) -> Self{
        let empty_grid_space: GridSpace  = GridSpace { building:None,direction:None };
         let mut default_map = vec![vec![empty_grid_space;251];251];
         //add any existing buildings before initial map build
         for (current_building_id, building) in building_hash_map{
            for y in building.bottom_left.1..building.bottom_left.1-building.height{
                for x in building.bottom_left.0..building.bottom_left.0+building.width{
                    default_map[x][y].building = Some(BuildingGridInfo {id: *current_building_id, typ: building.building_type })
                }
            }
         }
         Map{ map: default_map}
    }
    //TODO: add checks for if building is being built inside another building
    pub fn add_building(&mut self, building_id:u32, building:Building){
        for y in building.bottom_left.1..building.bottom_left.1-building.height{
            for x in building.bottom_left.0..building.bottom_left.0+building.width{
                self.map[x][y].building = Some(BuildingGridInfo {id: building_id, typ: building.building_type })
            }
        }
    }
    ///used to convert -500.0 to 500.0 to the grid system of 0 to 250 lossfully
    pub fn convert_position_to_grid_position(position:(f32,f32)) -> (usize,usize){
        let new_position:(f32,f32)= (((position.0+500.)/4.), ((position.1+500.)/4.));
        let output:(usize,usize)= (new_position.0.as_(), new_position.1.as_());
        println!("converted ({},{}) to ({},{}) to ({},{})",position.0,position.1,new_position.0,new_position.1,output.0,output.1);
        return output;
    }
    //Path system built into the grid system that priorizies nearest objective
    pub fn build_flow_path(&mut self, building_hash_map:&HashMap<u32,Building>){
        // do a level pathing off off each building position
        // steps:
        // build a double sidded queue
        println!("Path being built");
        let mut spread_queue:VecDeque<(usize,usize)> = VecDeque::new();
        // TODO: double check this beecause all of the paths seems to only have to points of orgins and also add it so there is a direction on this
        // feed the queue all outside layers of buildings, x lines take priority over corners to avoid wasted time with duplication
        for (_, building) in building_hash_map{
            // |__
            for y in building.bottom_left.1-1..building.bottom_left.1-building.height+1{
                spread_queue.push_back((building.bottom_left.0,y));
            }
            for x in building.bottom_left.0..building.bottom_left.0+building.width{
                spread_queue.push_back((x,building.bottom_left.1));
            }
            //  __
            //    |
            for y in building.bottom_left.1-1..building.bottom_left.1-building.height+1{
                spread_queue.push_back((building.bottom_left.0+building.width,y));
            }
            for x in building.bottom_left.0..building.bottom_left.0+building.width{
                spread_queue.push_back((x,building.bottom_left.1-building.height));
            }
        }
        // now MAKE IT SPREAD
        while !spread_queue.is_empty(){
            // pop the current space
            let current_location = spread_queue.pop_front().unwrap();
            // give all surrounding gridspaces that do not have (directions and building) directions to the current space
            for (index, direction) in DIRECTIONS.iter().enumerate(){
                //direction is off the grid
                if (direction.0 + current_location.0 as i32)<0 || (direction.0 + current_location.0 as i32) >250{
                    println!("fail 1");
                    continue;
                } 
                
                if (direction.1 + current_location.1 as i32) < 0 || (direction.1 + current_location.1 as i32) > 250{
                    println!("fail 2");
                    continue;
                }
                
                let current_surrounding_space_cord:(usize,usize)  = 
                    (usize::try_from(current_location.0 as i32+direction.0).unwrap(),usize::try_from(current_location.1 as i32+direction.1).unwrap());
                let current_surrounding_space:&mut GridSpace = 
                    &mut self.map[ current_surrounding_space_cord.0][current_surrounding_space_cord.1];
                if current_surrounding_space.building.is_none() && current_surrounding_space.direction.is_none(){
                    current_surrounding_space.direction = Some(Direction::new(index));
                    spread_queue.push_back(current_surrounding_space_cord);
                    println!("direction added")
                }
            }  
            //OR just feed the direction like i already set up to the directions that dont have a building and directions
        
        }
        // add the new spaces to the queue
        println!("Path finished being built");
    }
    // pub fn find_moveable_options(&self, x: i32, y: i32) -> Vec<(((i32, i32) u32))> {
    //     let movement_ammount = 5;
    //     let mut movement_options: Vec<((i32, i32),u32)> = vec![
    //         ((x + movement_ammount, y + movement_ammount),0),
    //         ((x + movement_ammount, y),0),
    //         ((x + movement_ammount, y - movement_ammount),0),
    //         ((x, y + movement_ammount),0),
    //         ((x, y - movement_ammount),0),
    //         ((x - movement_ammount, y + movement_ammount),0),
    //         ((x - movement_ammount, y),0),
    //         ((x - movement_ammount, y - movement_ammount),0),
    //     ];
    //     let mut index = 7;
    //     loop {
    //         let current_cord = movement_options[index].0;
    //         if current_cord.0 > 500
    //             || current_cord.1 > 500
    //             || current_cord.0 < -500
    //             || current_cord.1 < -500
    //         {
    //             // out of grid
    //         } else if self.map[(current_cord.0 + 500) as usize][(current_cord.1 + 500) as usize].is_some() {
    //             //need to it to alter the movement cost (+ for senteries, - for )
    //             movement_options.remove(index);
    //         }
    //         if index == 0 {
    //             break;
    //         }
    //         index -= 1;
    //     }
    //     movement_options
    // }
}
enum State {
    StartMenu,
    Playing,
    Paused,
    EndMenu,
}
struct MyGame {
    state: State,
    current_game: Game,
}
//TODO: ADD more details to add variance for other buildings besides barriers
struct Building{
    building_type: BuildingType,
    bottom_left: (usize,usize),
    width: usize,
    height: usize,
    max_health: f32,
    health: f32
}
struct Game {
    main_gun: main_gun::MainGun,
    map: Map,
    path_built:bool,
    last_building_added_id: u32,
    building_hash_map: HashMap<u32,Building>,
    enemy_alive_list: Vec<enemy::Enemy>,
    enemy_dead_list: Vec<enemy::Enemy>,
    worker_task_list: Vec<worker::Task>,
    worker_list: Vec<worker::Worker>,
    rooftop_view: bool,
    camera_zoom_ratio: f32,
    shake_meter: u8,
    camera_x: f32,
    camera_y: f32,
}
impl Default for Game {
    fn default() -> Self {
        //The health for the player/ main building/ main gun
            // width = 18
            // height =  12
            // bottom left x = 116
            // bottom left y = 119
        let mut building_hash_map:HashMap<u32,Building>= HashMap::new();
        let main_building = Building{ bottom_left: (134,131), width: 18, height: 12, max_health: 1000., health: 1000.,building_type:BuildingType::Sentry };
        building_hash_map.insert(0, main_building);
        Game {

            worker_list: Vec::new(),
            worker_task_list: Vec::new(),
            last_building_added_id: 0,
            map: Map::new(&building_hash_map),
            path_built: false,
            building_hash_map,
            main_gun: main_gun::MainGun {
                shooting_duration: 2.,
                enabled: true,
                rotation_speed_per_second: PI / 10.0,
                shell_explosive_radius: 50.,
                damage: 100.,
                ..Default::default()
            },
            enemy_alive_list: Vec::new(),
            enemy_dead_list: Vec::new(),
            rooftop_view: true,
            camera_zoom_ratio: 1.,
            shake_meter: 0,
            camera_x: -50.,
            camera_y: -50.,
        }
    }
}
///position used for defining a place for pathing
struct Pos {
    bottom_left: (i32,i32),
    scale: (i32,i32),
    center: (i32,i32)
}
impl Pos{
    fn new(bottom_left: (i32, i32),scale: (i32,i32)) -> Self{
        Pos{ bottom_left, scale, center: (bottom_left.0 + (scale.0/2),bottom_left.1 + (scale.1/2)) }
    }
    fn distance_to_center (&self, given_point:(i32,i32)) -> u32{
        self.center.0.abs_diff(given_point.0) + self.center.1.abs_diff(given_point.1)
    }
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        MyGame {
            state: State::StartMenu,
            current_game: Default::default(),
        }
    }
    /// Resets the variables in preperation for next game
    pub fn reset(&mut self) {
        self.state = State::StartMenu;
        self.current_game = Default::default();
    }
    fn add_building(){
        //TODO: ensure buildings dont overlap, use the id based on the last id in the game object +1
    }
    fn spawn_enemy(&mut self) -> Result<(), GameError> {
        let random_ratio: f32 = rand::thread_rng().gen_range(0.0..1.);
        let base_health: f32 = 100. * (random_ratio + 0.5);
        let base_size = 20. * (random_ratio + 0.5);
        let random_side: i8 = rand::thread_rng().gen_range(1..5);
        let mut random_side_length = 500. * random_ratio;
        if rand::random() {
            random_side_length = random_side_length * -1.;
        }
        println!("current side: {}", random_side);
        let position_generated: (f32, f32) = match random_side {
            1 => (-500., random_side_length),
            2 => (500., random_side_length),
            3 => (random_side_length, -500.),
            4 => (random_side_length, 500.),
            _ => {
                panic!()
            }
        };
        println!(
            "Enemy Spawned at {},{} with size {} with already {} enemies",
            &position_generated.0,
            &position_generated.1,
            &base_size,
            self.current_game.enemy_alive_list.len()
        );
        let new_enemy = Enemy {
            health: base_health,
            size: base_size,
            position: position_generated,
            speed: 15,
            rotation: 0.0,
            building_hit: None,
        };
        //Path has been moved onto the grid spaces
        //println!("Path building started");
        //new_enemy.path = self.build_path(position_generated, (0, 0)).unwrap().0;
        //println!("Path building finished");
        self.current_game.enemy_alive_list.push(new_enemy);
        Ok(())
    }
    //TODO optimize to the grid and not pizels so that workers can use this to reach dead bodies
    // fn build_path(
    //     &self,
    //     start_location: (i32, i32),
    //     goal_bottom_left: (i32, i32),
    //     goal_scale: (i32, i32)
    // ) -> Option<(Vec<(i32, i32)>, u32)> {
    //     let goal_limit = (goal_bottom_left.0 + goal_scale.0, goal_bottom_left.1 + goal_scale.1);
    //     let distance_x = 
    //     let result: Option<(Vec<(i32, i32)>, u32)> = astar(
    //         &start_location,
    //         |&(x, y)| {
    //             self.current_game
    //                 .map
    //                 .find_moveable_options(x, y)
    //         },
    //         |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
    //         |&(x,y)| x > ,
    //     );
    //     result
    // }
    fn offset_to_screen_cord(&self, ctx: &Context, screen_cord_wanted: &[f32; 2]) -> [f32; 2] {
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;
        let camera_world_view_width =
            DEFAULT_CAM_SIZE * self.current_game.camera_zoom_ratio * aspect_ratio;
        let camera_world_view_height = DEFAULT_CAM_SIZE * self.current_game.camera_zoom_ratio;
        let cord_wanted_x = screen_cord_wanted[0] * camera_world_view_width;
        let cord_wanted_y = screen_cord_wanted[1] * camera_world_view_height;
        let output_x = cord_wanted_x + (self.current_game.camera_x * aspect_ratio);
        let output_y = cord_wanted_y + self.current_game.camera_y;
        [output_x, output_y]
    }
    fn generate_shake_offset(&mut self) -> (f32, f32) {
        if self.current_game.shake_meter > 0 {
            let mut rng = rand::thread_rng();
            let current_shake_meter: f32 = self.current_game.shake_meter as f32;
            let gen_x: f32 = rng.gen();
            let gen_y: f32 = rng.gen();
            self.current_game.shake_meter -= 1;
            return (
                (gen_x * current_shake_meter) - (current_shake_meter / 2.),
                (gen_y * current_shake_meter) - (current_shake_meter / 2.),
            );
        } else {
            return (0., 0.);
        }
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
        let camera_world_view_width = DEFAULT_CAM_SIZE * self.current_game.camera_zoom_ratio;
        let camera_world_view_height = DEFAULT_CAM_SIZE * self.current_game.camera_zoom_ratio;
        let world_coord = [
            ndc[0] * camera_world_view_width * aspect_ratio
                + (self.current_game.camera_x * aspect_ratio),
            (ndc[1] * camera_world_view_height + self.current_game.camera_y) * -1.,
        ];

        println!("Converted {:?} to {:?}", screen_cord, world_coord);
        world_coord
    }
    fn switch_perspective(&mut self) -> Result<(), GameError> {
        self.current_game.rooftop_view = !self.current_game.rooftop_view;
        self.current_game.main_gun.enabled = self.current_game.rooftop_view;
        Ok(())
    }
    fn change_camera_zoom(&mut self, zoom_increase: bool) -> Result<(), GameError> {
        let diffrence_in_cam_size = DEFAULT_CAM_SIZE * 0.1;
        if zoom_increase {
            //increase zoom
            let new_camera_zoom_ratio = self.current_game.camera_zoom_ratio - 0.1;
            if new_camera_zoom_ratio > 0.5 {
                self.current_game.camera_zoom_ratio = new_camera_zoom_ratio;
                self.current_game.camera_x += diffrence_in_cam_size / 2.;
                self.current_game.camera_y += diffrence_in_cam_size / 2.;
            }
        } else {
            //decrease zoom
            self.current_game.camera_zoom_ratio += 0.1;
            self.current_game.camera_x -= diffrence_in_cam_size / 2.;
            self.current_game.camera_y -= diffrence_in_cam_size / 2.;
        }
        println!("Current Zoom:{}", self.current_game.camera_zoom_ratio);
        Ok(())
    }
    fn change_camera_location(&mut self, key_pressed: char) -> Result<(), GameError> {
        match key_pressed {
            'w' => self.current_game.camera_y -= 2. * self.current_game.camera_zoom_ratio,
            'a' => self.current_game.camera_x -= 2. * self.current_game.camera_zoom_ratio,
            's' => self.current_game.camera_y += 2. * self.current_game.camera_zoom_ratio,
            'd' => self.current_game.camera_x += 2. * self.current_game.camera_zoom_ratio,
            _ => (),
        }
        Ok(())
    }
    fn manage_enemies(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        if self.current_game.enemy_alive_list.len() != 0 {
            let mut current_enemy_index: usize = self.current_game.enemy_alive_list.len() - 1;
            //check enemies for abnomalities and spawn
            loop {
                let current_enemy = &mut self.current_game.enemy_alive_list[current_enemy_index];
                if current_enemy.health < 0. {
                    //put the dead enemies in the deadlist
                    self.current_game.enemy_dead_list.push(
                        self.current_game
                            .enemy_alive_list
                            .get(current_enemy_index)
                            .unwrap()
                            .clone(),
                    );
                    self.current_game
                        .enemy_alive_list
                        .remove(current_enemy_index);
                } else if current_enemy.draw_and_reach_base_check(ctx, canvas, &self.current_game.map) {
                    //despawn the ones that reached the base and apply dmg
                    //TODO: Despawn if reached any buildings as well
                    self.current_game.building_hash_map.get_mut(&current_enemy.building_hit.unwrap()).unwrap().health -= current_enemy
                        .health;
                    self.current_game
                        .enemy_alive_list
                        .remove(current_enemy_index);
                }
                if current_enemy_index == 0 {
                    break;
                }
                current_enemy_index -= 1;
            }
        }
        //draw dead enemies
        for enemy in &mut self.current_game.enemy_dead_list {
            enemy.draw_dead(ctx, canvas);
        }
    }
    fn draw_ui(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let uniform_og_scale = 30.0 * self.current_game.camera_zoom_ratio;
        let uniform_px_scale = PxScale::from(uniform_og_scale);
        let uniform_rescale = 0.1;
        //build health text
        let health_text_format = format!(
            "{} / {}",
            self.current_game.building_hash_map.get(&0).unwrap().health as i32,
            self.current_game.building_hash_map.get(&0).unwrap().max_health as i32
        );
        let mut health_text_fragment = TextFragment::new(health_text_format);
        health_text_fragment.color = Some(Color::WHITE);
        health_text_fragment.scale = Some(uniform_px_scale);
        let health_text = Text::new(health_text_fragment);
        //let health_text_offset = self.current_game.offset_to_screen_cord(ctx, &[0.05, 0.03]);
        let health_screen_offset = self.offset_to_screen_cord(ctx, &[0.01, 0.01]);
        //build health bar
        let health_bar_border_size = 1.5 * self.current_game.camera_zoom_ratio;
        let health_size: [f32; 2] = [
            30. * self.current_game.camera_zoom_ratio,
            7. * self.current_game.camera_zoom_ratio,
        ];
        let percent_health =
        self.current_game.building_hash_map.get(&0).unwrap().health / self.current_game.building_hash_map.get(&0).unwrap().max_health;
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
        if self.current_game.worker_task_list.len() > 0 {
            let mut current_worker_index: usize = self.current_game.worker_list.len() - 1;

            loop {
                let current_worker = &mut self.current_game.worker_list[current_worker_index];
                if current_worker.health < 0. {
                    self.current_game.worker_list.remove(current_worker_index);
                } else {
                    if current_worker.avalible_for_task {
                        if self.current_game.worker_task_list.len() != 0 {
                            current_worker.task = self.current_game.worker_task_list[0].clone();
                            current_worker.time_since_path_started =
                                ctx.time.time_since_start().as_secs_f32();
                            let start_location: (i32, i32) = current_worker.position;
                            let goal_location: (f32, f32) = self.current_game.worker_task_list[0].goals[0];
                            current_worker.avalible_for_task = false;
                            
                            //TODO: reimplement ASTAR in worker
                            //Will have to optimize path builder so that a worker will be able to reach the dead bodies against the flow path system
                            //let path = self.build_path(start_location, goal_location).unwrap().0;
                            let current_worker =
                                &mut self.current_game.worker_list[current_worker_index];
                            //current_worker.path = path;
                            self.current_game.worker_task_list.remove(0);
                        }
                    } else if current_worker.ready_for_new_path {
                    }
                    let current_worker = &mut self.current_game.worker_list[current_worker_index];
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
        if !self.current_game.rooftop_view {
            for bad_guy in &self.current_game.enemy_dead_list {
                let current_hitbox = bad_guy.get_hitbox();
                if mouse_x > current_hitbox.bottom_left.0
                    && mouse_x < current_hitbox.bottom_left.0 + current_hitbox.width
                    && mouse_y > current_hitbox.bottom_left.1
                    && mouse_y < current_hitbox.bottom_left.1 + current_hitbox.height
                {
                    // a dead enemy was clicked
                    let time_to_collect_body: f32 = 0.5;
                    let time_to_deposit_body: f32 = 0.5;
                    let home_cord: (f32, f32) = (0., 0.);
                    let collect_dead_task = worker::Task {
                        task_times: vec![time_to_collect_body, time_to_deposit_body],
                        goals: vec![bad_guy.position, home_cord],
                    };
                    self.current_game.worker_task_list.push(collect_dead_task);
                    break;
                }
            }
        }
    }

    fn draw_end_menu(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        //game over text
        let mut over_text_fragment = TextFragment::new("HOPE IS LOST");
        over_text_fragment.color = Some(Color::RED);
        over_text_fragment.scale = Some(PxScale::from(0.1 * window_size.height as f32));
        let over_text = Text::new(over_text_fragment);
        over_text.draw(
            &mut canvas,
            DrawParam::default().dest([
                (window_size.width as f32 - over_text.measure(&ctx.gfx).unwrap().x) / 2.,
                0.025 * window_size.height as f32,
            ]),
        );
        //States outline
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::BLACK)
                .scale([
                    0.7 * window_size.width as f32,
                    0.7 * window_size.height as f32,
                ])
                .dest([
                    0.15 * window_size.width as f32,
                    0.15 * window_size.height as f32 as f32,
                ]),
        );
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::WHITE)
                .scale([
                    0.60 * window_size.width as f32,
                    0.64 * window_size.height as f32,
                ])
                .dest([
                    0.20 * window_size.width as f32,
                    0.18 * window_size.height as f32 as f32,
                ]),
        );
        //Stats
        //TODO: add important stats

        //Play again text
        let mut play_text_fragment = TextFragment::new("FIGHT ONCE MORE");
        play_text_fragment.color = Some(Color::GREEN);
        play_text_fragment.scale = Some(PxScale::from(0.1 * window_size.height as f32));
        let play_text = Text::new(play_text_fragment);
        play_text.draw(
            &mut canvas,
            DrawParam::default().dest([
                (window_size.width as f32 - play_text.measure(&ctx.gfx).unwrap().x) / 2.,
                0.875 * window_size.height as f32,
            ]),
        );
        canvas.finish(ctx)
    }
    fn draw_start_menu(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        //DRAW BARS Horiz
        let mut current_bar_start = 0.00;
        while current_bar_start < 1. {
            current_bar_start += 0.10;
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::BLACK)
                    .scale([0.05 * window_size.width as f32, window_size.height as f32])
                    .dest([current_bar_start * window_size.width as f32, 0. as f32]),
            );
        }
        //DRAW BARS Vertical
        current_bar_start = 0.00;
        while current_bar_start < 1. {
            current_bar_start += 0.1;
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::BLACK)
                    .scale([window_size.width as f32, 0.05 * window_size.height as f32])
                    .dest([0. as f32, current_bar_start * window_size.height as f32]),
            );
        }
        //DRAW BUTTON BACKING
        canvas.draw(
            &ggez::graphics::Quad,
            DrawParam::default()
                .color(Color::BLACK)
                .scale([
                    0.41 * window_size.width as f32,
                    0.25 * window_size.height as f32,
                ])
                .dest([
                    0.27 * window_size.width as f32,
                    0.35 * window_size.height as f32,
                ]),
        );
        //DRAW BUTTON TEXT
        let mut play_text_fragment = TextFragment::new("PLAY");
        play_text_fragment.color = Some(Color::GREEN);
        play_text_fragment.scale = Some(PxScale {
            x: (0.15 * window_size.width as f32),
            y: (0.15 * window_size.height as f32),
        });
        let play_text = Text::new(play_text_fragment);
        play_text.draw(
            &mut canvas,
            DrawParam::default().dest([
                0.32 * window_size.width as f32,
                0.40 * window_size.height as f32,
            ]),
        );
        canvas.finish(ctx)
    }
    fn draw_playing(&mut self, ctx: &mut Context) -> GameResult {
        if self.current_game.building_hash_map.get(&0).unwrap().health < 0. {
            self.state = State::EndMenu;
        }
        if !self.current_game.path_built{
            self.current_game.map.build_flow_path(&self.current_game.building_hash_map);
            self.current_game.path_built = true;
        }
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let window = ctx.gfx.window();
        let window_size = window.inner_size();
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;
        let shake = self.generate_shake_offset();
        canvas.set_screen_coordinates(Rect::new(
            (self.current_game.camera_x + shake.0) * aspect_ratio,
            self.current_game.camera_y + shake.1,
            (DEFAULT_CAM_SIZE * aspect_ratio) * self.current_game.camera_zoom_ratio,
            DEFAULT_CAM_SIZE * self.current_game.camera_zoom_ratio,
        ));
        if self.current_game.rooftop_view {
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
            self.current_game
                .main_gun
                .draw_ammo_loader(ctx, &mut canvas);
            self.current_game.main_gun.draw(
                ctx,
                &mut canvas,
                &mut self.current_game.enemy_alive_list,
                &mut self.current_game.shake_meter,
            );
        } else {
            // draw ground scene
            //the floor
            //TODO: make this turrent location more readily converted to grid
            //converted by doing (x+500)/4 or if specifying distance x/4
            // width = 18
            // height =  12
            // bottom left x = 116
            // bottom left y = 119
            canvas.draw(
                &ggez::graphics::Quad,
                DrawParam::default()
                    .color(Color::from_rgb(128, 128, 128))
                    .scale([72., 48.])
                    .dest([-36., -24.]),
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

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // put logic in the drawing function, will change in future need be
        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if matches!(self.state, State::Playing) {
            match _button {
                MouseButton::Left => {
                    // creates new Circle and push to vector
                    let [x, y] = self.screen_cord_to_world_cord(ctx, &[x, y]);
                    self.current_game.main_gun.initiate_fire_sequence(x, y);
                }
                MouseButton::Right => {
                    // creates new Circle and push to vector
                    let [x, y] = self.screen_cord_to_world_cord(ctx, &[x, y]);
                    self.initiate_task(x, y);
                }
                _ => {}
            }
        } else if matches!(self.state, State::StartMenu) {
            match _button {
                MouseButton::Left => {
                    self.state = State::Playing;
                }
                _ => {}
            }
        } else if matches!(self.state, State::EndMenu) {
            match _button {
                MouseButton::Left => {
                    self.reset();
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        if matches!(self.state, State::Playing) {
            match input.keycode {
                Some(VirtualKeyCode::Tab) => self.switch_perspective(),
                Some(VirtualKeyCode::Q) => self.change_camera_zoom(false),
                Some(VirtualKeyCode::E) => self.change_camera_zoom(true),
                Some(VirtualKeyCode::W) => self.change_camera_location('w'),
                Some(VirtualKeyCode::A) => self.change_camera_location('a'),
                Some(VirtualKeyCode::S) => self.change_camera_location('s'),
                Some(VirtualKeyCode::D) => self.change_camera_location('d'),
                Some(VirtualKeyCode::Z) => self.spawn_enemy(),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        match self.state {
            State::StartMenu => self.draw_start_menu(ctx),
            State::Playing => self.draw_playing(ctx),
            State::Paused => Ok(()),
            State::EndMenu => self.draw_end_menu(ctx),
        }
    }
}
