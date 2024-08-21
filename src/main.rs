use ::rand::prelude::*;
use macroquad::prelude::*;
use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
    time::{Duration, Instant},
};

const N: usize = 2;
const CELLS_ROWS: LazyLock<RwLock<usize>> = LazyLock::new(|| RwLock::new(2));
const SCREEN_WIDTH: f32 = 1920.0;
const SCREEN_HEIGHT: f32 = 1080.0;
pub static CELL_HEIGHT: LazyLock<f32> =
    LazyLock::new(|| SCREEN_HEIGHT / *CELLS_ROWS.read().unwrap() as f32);

pub static CELLS_COLUMNS: LazyLock<RwLock<usize>> = LazyLock::new(|| {
    RwLock::new((*CELLS_ROWS.read().unwrap() as f32 * (SCREEN_WIDTH / SCREEN_HEIGHT)) as usize)
});

pub static CELL_WIDTH: LazyLock<f32> =
    LazyLock::new(|| SCREEN_WIDTH / *CELLS_COLUMNS.read().unwrap() as f32);

#[derive(Clone, Debug)]
struct Object {
    pos: Vec2,
    color: Color,
}

impl Object {
    fn find_closest<'a>(
        &self,
        id: &'a Instant,
        objects: &'a [Vec<HashMap<Instant, Self>>],
    ) -> &'a Self {
        let initial_cell = cell_by_pos(self.pos);

        let mut directions = ivec4(
            initial_cell.0,
            initial_cell.0,
            initial_cell.1,
            initial_cell.1,
        );

        let mut visible_objects = Vec::new();

        for (object_id, object) in &objects[initial_cell.1 as usize][initial_cell.0 as usize] {
            if object_id != id {
                visible_objects.push(object);
            }
        }

        let starting_visible_objects = visible_objects.len();

        loop {
            directions.x = (directions.x - 1).max(0);
            directions.y = (directions.y + 1).min(*CELLS_COLUMNS.read().unwrap() as i32 - 1);

            directions.z = (directions.z - 1).max(0);
            directions.w = (directions.w + 1).min(*CELLS_ROWS.read().unwrap() as i32 - 1);

            for row in directions.z..=directions.w {
                for column in directions.x..=directions.y {
                    if (column, row) != initial_cell {
                        for object in objects[row as usize][column as usize].values() {
                            visible_objects.push(object);
                        }
                    }
                }
            }

            let first_step_closest_object = visible_objects.iter().min_by(|a, b| {
                self.pos
                    .distance(a.pos)
                    .partial_cmp(&self.pos.distance(b.pos))
                    .unwrap()
            });

            if starting_visible_objects > 0 && starting_visible_objects == visible_objects.len() {
                return first_step_closest_object.unwrap();
            }

            if starting_visible_objects != visible_objects.len() {
                let mut visible_objects_new = Vec::new();

                let r = self.pos.distance(first_step_closest_object.unwrap().pos);

                let min_x = (((self.pos.x - r) / *CELL_WIDTH) as usize).max(0);
                let max_x = (((self.pos.x + r) / *CELL_WIDTH) as usize)
                    .min(*CELLS_COLUMNS.read().unwrap() - 1);
                let min_y = (((self.pos.y - r) / *CELL_HEIGHT) as usize).max(0);
                let max_y = (((self.pos.y + r) / *CELL_HEIGHT) as usize)
                    .min(*CELLS_ROWS.read().unwrap() - 1);

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        for (object_id, object) in &objects[y][x] {
                            if object_id != id {
                                visible_objects_new.push(object);
                            }
                        }
                    }
                }

                return visible_objects_new
                    .iter()
                    .min_by(|a, b| {
                        self.pos
                            .distance(a.pos)
                            .partial_cmp(&self.pos.distance(b.pos))
                            .unwrap()
                    })
                    .unwrap();
            }
        }
    }
}

fn cell_by_pos(pos: Vec2) -> (i32, i32) {
    ((pos.x / *CELL_WIDTH) as i32, (pos.y / *CELL_HEIGHT) as i32)
}

fn reset_objects(objects: &mut [Vec<HashMap<Instant, Object>>], rng: &mut StdRng) {
    for row in &mut *objects {
        for column in row {
            column.clear();
        }
    }

    for _ in 0..N {
        let pos = vec2(
            rng.gen_range(0.0..SCREEN_WIDTH),
            rng.gen_range(0.0..SCREEN_HEIGHT),
        );
        let (cell_x, cell_y) = cell_by_pos(pos);

        objects[cell_y as usize][cell_x as usize]
            .insert(Instant::now(), Object { pos, color: GREEN });
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut rng = StdRng::from_rng(&mut thread_rng()).unwrap();
    // A workaround for Linux
    if cfg!(target_os = "linux") {
        set_fullscreen(true);
        std::thread::sleep(std::time::Duration::from_secs(1));
        next_frame().await;
    }

    let mut objects: Vec<Vec<HashMap<Instant, Object>>> =
        vec![vec![HashMap::new(); *CELLS_COLUMNS.read().unwrap()]; *CELLS_ROWS.read().unwrap()];

    let mut lines: Vec<(Vec2, Vec2)> = Vec::new();
    let mut timer: Option<Duration> = None;

    for _ in 0..N {
        let pos = vec2(
            rng.gen_range(0.0..SCREEN_WIDTH),
            rng.gen_range(0.0..SCREEN_HEIGHT),
        );
        let (cell_x, cell_y) = cell_by_pos(pos);

        objects[cell_y as usize][cell_x as usize]
            .insert(Instant::now(), Object { pos, color: GREEN });
    }

    loop {
        if is_key_pressed(KeyCode::Key2) {
            lines.clear();
        }

        if is_key_pressed(KeyCode::Key3) {
            reset_objects(&mut objects, &mut rng);

            lines.clear();
        }

        for (lhs, rhs) in &lines {
            draw_line(lhs.x, lhs.y, rhs.x, rhs.y, 2.0, WHITE);
        }

        if is_key_pressed(KeyCode::Key1) {
            let timestamp = Instant::now();

            for row in &objects {
                for column in row {
                    for (object_id, object) in column {
                        let closest = object.find_closest(object_id, &objects);

                        lines.push((object.pos, closest.pos));
                    }
                }
            }

            timer = Some(timestamp.elapsed());
        }

        for row in 0..*CELLS_ROWS.read().unwrap() {
            for column in 0..*CELLS_COLUMNS.read().unwrap() {
                for object in objects[row][column].values() {
                    draw_circle(object.pos.x, object.pos.y, 5.0, object.color);
                }

                draw_rectangle_lines(
                    column as f32 * *CELL_WIDTH,
                    row as f32 * *CELL_HEIGHT,
                    *CELL_WIDTH,
                    *CELL_HEIGHT,
                    1.0,
                    BLUE,
                );
            }
        }

        if let Some(timer) = timer {
            let text = &format!("{:?}ns", timer.as_nanos());
            let measured = measure_text(text, None, 50, 1.0);

            draw_rectangle(
                SCREEN_WIDTH / 2.0 - measured.width / 2.0,
                measured.offset_y - 5.0,
                measured.width,
                measured.offset_y + 10.0,
                WHITE,
            );

            draw_text(
                text,
                SCREEN_WIDTH / 2.0 - measured.width / 2.0,
                measured.height * 2.0,
                50.0,
                BLACK,
            );
        }

        next_frame().await
    }
}
