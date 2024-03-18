use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct Maze {
    // TODO make this bit-packed booleans that represent the walls
    cells: Box<[Cell]>,
    pub width: u32,
    pub height: u32,
}

impl Maze {
    pub fn new(width: u32, height: u32) -> Self {
        let len = width * height;
        let cells = vec![Cell::new(false, false); usize::try_from(len).unwrap()].into_boxed_slice();
        Self {
            cells,
            width,
            height,
        }
    }

    pub fn cell_mut(&mut self, index: usize) -> &mut Cell {
        &mut self.cells[index]
    }

    pub fn draw(&self, cell_size: u32) -> Pixmap {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 200);
        paint.anti_alias = true;

        let stroke = Stroke::default();

        let width = self.width * cell_size;
        let height = self.height * cell_size;
        let mut pixmap = Pixmap::new(width, height).unwrap();

        let transform = Transform::identity();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = usize::try_from(y * self.width + x).unwrap();
                let cell = &self.cells[idx];

                if cell.north_open() && cell.west_open() {
                    continue;
                }

                let top_left_x = f64::from(x * cell_size);
                let top_left_y = f64::from(y * cell_size);

                // Leave a gap for the entrance at the north wall of 0,0
                if !(cell.north_open() || x == 0 && y == 0) {
                    let mut pb = PathBuilder::new();
                    pb.move_to(top_left_x as f32, top_left_y as f32);
                    pb.line_to(
                        (top_left_x + f64::from(cell_size)) as f32,
                        top_left_y as f32,
                    );
                    let path = pb.finish().unwrap();
                    pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                }

                if !cell.west_open() {
                    let mut pb = PathBuilder::new();
                    pb.move_to(top_left_x as f32, top_left_y as f32);
                    pb.line_to(
                        top_left_x as f32,
                        (top_left_y + f64::from(cell_size)) as f32,
                    );
                    let path = pb.finish().unwrap();
                    pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                }
            }
        }

        // Southern-most wall
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, f64::from(self.height * cell_size) as f32);
        pb.line_to(
            // Leave a gap for the exit
            f64::from((self.width - 1) * cell_size) as f32,
            f64::from(self.height * cell_size) as f32,
        );
        let path = pb.finish().unwrap();
        pixmap.stroke_path(&path, &paint, &stroke, transform, None);

        // Eastern-most wall
        let mut pb = PathBuilder::new();
        pb.move_to(f64::from(self.width * cell_size) as f32, 0.0);
        pb.line_to(
            f64::from(self.width * cell_size) as f32,
            f64::from(self.height * cell_size) as f32,
        );
        let path = pb.finish().unwrap();
        pixmap.stroke_path(&path, &paint, &stroke, transform, None);

        pixmap
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    bits: u8,
}

impl Cell {
    pub fn new(west_open: bool, north_open: bool) -> Self {
        let bits = (if west_open { 1u8 } else { 0u8 }) | (if north_open { 2u8 } else { 0u8 });
        Cell { bits }
    }

    pub fn west_open(&self) -> bool {
        self.bits & 1u8 > 0
    }

    pub fn set_west_open(&mut self) {
        self.bits |= 1u8;
    }

    pub fn north_open(&self) -> bool {
        self.bits & 2u8 > 0
    }

    pub fn set_north_open(&mut self) {
        self.bits |= 2u8;
    }
}
