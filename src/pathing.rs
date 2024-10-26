use pathfinding::prelude::astar;

pub fn build_path_star(
    &self,
    start_location: (i32, i32),
    goal_bottom_left: (i32, i32),
    goal_scale: (i32, i32)
) -> Option<(Vec<(i32, i32)>, u32)> {
    let goal_limit = (goal_bottom_left.0 + goal_scale.0, goal_bottom_left.1 + goal_scale.1);
    let distance_x = 
    let result: Option<(Vec<(i32, i32)>, u32)> = astar(
        &start_location,
        |&(x, y)| {
            self.current_game
                .map
                .clone()
                .find_moveable_options(x, y)
                .into_iter()
                .map(|p| (p, 1))
        },
        |&(x, y)| (goal.0.abs_diff(x) + goal.1.abs_diff(y)) / 3,
        |&(x,y)| ,
    );
    result
}