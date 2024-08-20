use ::rand::prelude::*;
use macroquad::prelude::*;
use std::{collections::HashMap, sync::LazyLock, time::Instant};

const N: usize = 300;
const CELLS_ROWS: usize = 9;
const SCREEN_WIDTH: f32 = 1920.0;
const SCREEN_HEIGHT: f32 = 1080.0;
pub static CELL_HEIGHT: LazyLock<f32> = LazyLock::new(|| SCREEN_HEIGHT / CELLS_ROWS as f32);

pub static CELLS_COLUMNS: LazyLock<usize> =
    LazyLock::new(|| (CELLS_ROWS as f32 * (SCREEN_WIDTH / SCREEN_HEIGHT)) as usize);

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
    ) -> Option<&'a Self> {
        let initial_cell = cell_by_pos(self.pos);

        let mut delta = 0;
        let mut left = initial_cell;
        let mut right = initial_cell;
        let mut up = initial_cell;
        let mut down = initial_cell;

        let mut visible_objects = HashMap::new();

        for (object_id, object) in &objects[initial_cell.1 as usize][initial_cell.0 as usize] {
            visible_objects.insert(object_id, object);
        }

        visible_objects.remove(&id);

        let mut new_added = false;

        let mut last_directions = (left, right, up, down);

        loop {
            delta += 1;
            left.0 = (left.0 - delta).max(0);
            right.0 = (right.0 + delta).min(*CELLS_COLUMNS as isize - 1);

            up.1 = (up.1 - delta).max(0);
            down.1 = (down.1 + delta).min(CELLS_ROWS as isize - 1);

            if (left, right, up, down) == last_directions {
                return None;
            } else {
                last_directions = (left, right, up, down);
            }

            for direction in [left, right, up, down] {
                if direction != initial_cell {
                    for (object_id, object) in &objects[direction.1 as usize][direction.0 as usize]
                    {
                        new_added = true;
                        visible_objects.insert(object_id, object);
                    }
                }
            }

            for row in initial_cell.1..=down.1 {
                for column in left.0..=initial_cell.0 {
                    if (column, row) != initial_cell {
                        for (object_id, object) in &objects[row as usize][column as usize] {
                            new_added = true;
                            visible_objects.insert(object_id, object);
                        }
                    }
                }
            }

            for row in initial_cell.1..=down.1 {
                for column in initial_cell.0..=right.0 {
                    if (column, row) != initial_cell {
                        for (object_id, object) in &objects[row as usize][column as usize] {
                            new_added = true;
                            visible_objects.insert(object_id, object);
                        }
                    }
                }
            }

            for row in up.1..=initial_cell.1 {
                for column in left.0..=initial_cell.0 {
                    if (column, row) != initial_cell {
                        for (object_id, object) in &objects[row as usize][column as usize] {
                            new_added = true;
                            visible_objects.insert(object_id, object);
                        }
                    }
                }
            }

            for row in up.1..=initial_cell.1 {
                for column in initial_cell.0..=right.0 {
                    if (column, row) != initial_cell {
                        for (object_id, object) in &objects[row as usize][column as usize] {
                            new_added = true;
                            visible_objects.insert(object_id, object);
                        }
                    }
                }
            }

            if new_added {
                return Some(visible_objects.values().min_by(|a, b| {
                    self.pos
                        .distance(a.pos)
                        .partial_cmp(&self.pos.distance(b.pos))
                        .unwrap()
                })?);
            }
        }
    }
}

fn cell_by_pos(pos: Vec2) -> (isize, isize) {
    (
        (pos.x / *CELL_WIDTH) as isize,
        (pos.y / *CELL_HEIGHT) as isize,
    )
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
        vec![vec![HashMap::new(); *CELLS_COLUMNS]; CELLS_ROWS];

    let mut lines: Vec<(Vec2, Vec2)> = Vec::new();

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

        for (lhs, rhs) in &lines {
            draw_line(lhs.x, lhs.y, rhs.x, rhs.y, 5.0, WHITE);
        }

        for row in 0..CELLS_ROWS {
            for column in 0..*CELLS_COLUMNS {
                draw_rectangle_lines(
                    column as f32 * *CELL_WIDTH,
                    row as f32 * *CELL_HEIGHT,
                    *CELL_WIDTH,
                    *CELL_HEIGHT,
                    5.0,
                    GREEN,
                );

                for (object_id, object) in &objects[row][column] {
                    if is_key_down(KeyCode::Key1) {
                        if let Some(closest) = object.find_closest(object_id, &objects) {
                            lines.push((object.pos, closest.pos));
                        }
                    }
                    draw_circle(object.pos.x, object.pos.y, 5.0, YELLOW);
                }
            }
        }

        next_frame().await
    }
}
