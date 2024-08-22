use ::rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use macroquad::prelude::*;

use std::{
    collections::HashMap,
    ops::RangeInclusive,
    sync::LazyLock,
    time::{Duration, Instant},
};

const OBJECT_RADIUS: f32 = 5.0;
const COLOR_RANGE: RangeInclusive<u8> = 30..=255;

const N: u64 = 1000;
const CELLS_ROWS: u64 = 85;
const SCREEN_WIDTH: f32 = 1920.0;
const SCREEN_HEIGHT: f32 = 1080.0;

pub static CELL_HEIGHT: f32 = SCREEN_HEIGHT / CELLS_ROWS as f32;
pub static CELLS_COLUMNS: LazyLock<u64> =
    LazyLock::new(|| (CELLS_ROWS as f32 * (SCREEN_WIDTH / SCREEN_HEIGHT)).round() as u64);
pub static CELL_WIDTH: LazyLock<f32> = LazyLock::new(|| SCREEN_WIDTH / *CELLS_COLUMNS as f32);

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

        let cell_pos = u64vec2(initial_cell.0, initial_cell.1);

        let mut layer = 0;

        let mut visible_objects = Vec::new();

        loop {
            // Top x
            if cell_pos.y >= layer {
                for x in
                    cell_pos.x.saturating_sub(layer)..=(cell_pos.x + layer).min(*CELLS_COLUMNS - 1)
                {
                    for (object_id, object) in &objects[(cell_pos.y - layer) as usize][x as usize] {
                        if object_id != id {
                            visible_objects.push(object);
                        }
                    }
                }
            }

            // Bottom x
            if layer > 0 && cell_pos.y + layer <= CELLS_ROWS - 1 {
                for x in
                    cell_pos.x.saturating_sub(layer)..=(cell_pos.x + layer).min(*CELLS_COLUMNS - 1)
                {
                    for object in objects[(cell_pos.y + layer) as usize][x as usize].values() {
                        visible_objects.push(object);
                    }
                }
            }

            // Left y
            if layer > 0 && cell_pos.x >= layer {
                for y in cell_pos.y.saturating_sub(layer - 1)
                    ..=(cell_pos.y + (layer - 1)).min(CELLS_ROWS - 1)
                {
                    for object in objects[y as usize][(cell_pos.x - layer) as usize].values() {
                        visible_objects.push(object);
                    }
                }
            }

            // Right y
            if layer > 0 && cell_pos.x + layer <= *CELLS_COLUMNS - 1 {
                for y in cell_pos.y.saturating_sub(layer - 1)
                    ..=(cell_pos.y + (layer - 1)).min(CELLS_ROWS - 1)
                {
                    for object in objects[y as usize][(cell_pos.x + layer) as usize].values() {
                        visible_objects.push(object);
                    }
                }
            }

            if !visible_objects.is_empty() {
                break;
            }

            layer += 1;
        }

        let first_step_closest_object = visible_objects
            .iter()
            .min_by(|a, b| {
                self.pos
                    .distance(a.pos)
                    .partial_cmp(&self.pos.distance(b.pos))
                    .unwrap()
            })
            .unwrap();

        let mut visible_objects_new = Vec::new();

        visible_objects_new.push(*first_step_closest_object);

        let r = self.pos.distance(first_step_closest_object.pos);

        let min_x = ((self.pos.x - r) / *CELL_WIDTH).max(0.0) as u64;
        let max_x = (((self.pos.x + r) / *CELL_WIDTH) as u64).min(*CELLS_COLUMNS - 1);

        let min_y = ((self.pos.y - r) / CELL_HEIGHT).max(0.0) as u64;
        let max_y = (((self.pos.y + r) / CELL_HEIGHT) as u64).min(CELLS_ROWS - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !(initial_cell.0.saturating_sub(layer)
                    ..=(initial_cell.0 + layer).min(*CELLS_COLUMNS - 1))
                    .contains(&x)
                    && !(initial_cell.1.saturating_sub(layer)
                        ..=(initial_cell.1 + layer).min(CELLS_ROWS - 1))
                        .contains(&y)
                {
                    for object in objects[y as usize][x as usize].values() {
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

fn cell_by_pos(pos: Vec2) -> (u64, u64) {
    ((pos.x / *CELL_WIDTH) as u64, (pos.y / CELL_HEIGHT) as u64)
}

fn reset_objects(objects: &mut [Vec<HashMap<Instant, Object>>]) {
    for row in &mut *objects {
        for column in row {
            column.clear();
        }
    }
}

fn spawn_objects(objects: &mut [Vec<HashMap<Instant, Object>>], rng: &mut StdRng) {
    for _ in 0..N {
        let pos = vec2(
            rng.gen_range(0.0..SCREEN_WIDTH),
            rng.gen_range(0.0..SCREEN_HEIGHT),
        );
        let (cell_x, cell_y) = cell_by_pos(pos);

        let color = Color::from_rgba(
            rng.gen_range(COLOR_RANGE),
            rng.gen_range(COLOR_RANGE),
            rng.gen_range(COLOR_RANGE),
            255,
        );
        objects[cell_y as usize][cell_x as usize].insert(Instant::now(), Object { pos, color });
    }
}

#[macroquad::main("spatial hashing")]
async fn main() {
    let mut rng = StdRng::from_rng(&mut thread_rng()).unwrap();
    // A workaround for Linux
    if cfg!(target_os = "linux") {
        set_fullscreen(true);
        std::thread::sleep(std::time::Duration::from_secs(1));
        next_frame().await;
    }

    let mut objects: Vec<Vec<HashMap<Instant, Object>>> =
        vec![vec![HashMap::new(); *CELLS_COLUMNS as usize]; CELLS_ROWS as usize];

    spawn_objects(&mut objects, &mut rng);

    let mut lines: Vec<(Vec2, Vec2, Color)> = Vec::new();
    let mut timer: Option<Duration> = None;

    loop {
        if is_key_pressed(KeyCode::Key2) {
            lines.clear();
        }

        if is_key_pressed(KeyCode::Key3) {
            reset_objects(&mut objects);
            spawn_objects(&mut objects, &mut rng);

            lines.clear();
        }

        for (lhs_point, rhs_point, color) in &lines {
            draw_line(
                lhs_point.x,
                lhs_point.y,
                rhs_point.x,
                rhs_point.y,
                2.0,
                *color,
            );
        }

        if is_key_pressed(KeyCode::Key1) {
            let timestamp = Instant::now();

            for row in &objects {
                for column in row {
                    for (object_id, object) in column {
                        let closest = object.find_closest(object_id, &objects);

                        lines.push((object.pos, closest.pos, object.color));
                    }
                }
            }

            timer = Some(timestamp.elapsed());
        }

        for (row_index, row) in objects.iter().enumerate() {
            for (column_index, column) in row.iter().enumerate() {
                for object in column.values() {
                    draw_circle(object.pos.x, object.pos.y, OBJECT_RADIUS, object.color);
                }

                draw_rectangle_lines(
                    column_index as f32 * *CELL_WIDTH,
                    row_index as f32 * CELL_HEIGHT,
                    *CELL_WIDTH,
                    CELL_HEIGHT,
                    1.0,
                    BLUE,
                );
            }
        }

        if let Some(timer) = timer {
            let text = &format!("{}s", timer.as_secs_f64());
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
