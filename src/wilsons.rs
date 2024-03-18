use std::fmt::{Display, Write};

use rand::prelude::*;

use crate::maze::{Cell as MazeCell, Maze};

#[derive(Debug)]
pub struct Generator {
    cells: Box<[Cell]>,
    pub width: usize,
    pub height: usize,
    unvisited_candidates: Vec<usize>,
}

impl Generator {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        let cells = vec![Cell::Empty; len].into_boxed_slice();
        let unvisited_candidates: Vec<usize> = (0..len).collect();
        Self {
            cells,
            width,
            height,
            unvisited_candidates,
        }
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    #[must_use]
    fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    fn cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    fn cell_mut(&mut self, index: usize) -> &mut Cell {
        &mut self.cells[index]
    }

    fn adjacent_index(&self, index: usize, direction: Direction) -> Option<usize> {
        use Direction::*;
        match direction {
            West if index % self.width == 0 => None,
            West => Some(index - 1),
            East if index % self.width == self.width - 1 => None,
            East => Some(index + 1),
            North if index < self.width => None,
            North => Some(index - self.width),
            South if index >= self.width * (self.height - 1) => None,
            South => Some(index + self.width),
        }
    }

    pub fn generate<R: Rng>(mut self, rng: &mut R) -> Maze {
        use Cell::*;
        use Direction::*;
        if self.is_empty() {
            return Maze::new(0, 0);
        }
        self.unvisited_candidates.shuffle(rng);

        // Choose an initial cell at random to be part of the maze
        let initial_idx = self.choose_walk_start().unwrap();
        *self.cell_mut(initial_idx) = InMaze(MazeCell::new(false, false));

        let mut directions = [North, South, East, West];
        let mut walk_indexes = Vec::with_capacity(self.len());
        while let Some(start_idx) = self.choose_walk_start() {
            walk_indexes.clear();
            let mut curr_idx = start_idx;

            // Perform the walk
            loop {
                walk_indexes.push(curr_idx);

                let (direction, adjacent_idx) =
                    self.choose_random_adjacent(curr_idx, &mut directions, rng);

                *self.cell_mut(curr_idx) = Walk(direction);
                if let InMaze(_) = self.cell(adjacent_idx) {
                    break;
                }
                curr_idx = adjacent_idx;
            }

            // Add the walk to the maze. Because the path may intersect itself and directions in
            // the cells that are traversed multiple times are overridden, follow the path based on
            // the directions not the indexes traversed. That means that some cells traversed in
            // the walk will not be included in the maze in this.
            curr_idx = start_idx;
            let mut last_direction: Option<Direction> = None;
            loop {
                let cell = self.cell_mut(curr_idx);
                let direction = match cell {
                    Walk(direction) => *direction,
                    InMaze(mc) => {
                        // Open up the existing maze cell so that the walk path enters it
                        match last_direction {
                            Some(East) => mc.set_west_open(),
                            Some(South) => mc.set_north_open(),
                            _ => {}
                        }
                        break;
                    }
                    _ => unreachable!(),
                };
                // Open up walls along the walk direction. Maze cells "own" the north and west
                // direction, so track if those are the walls entered from or leaving through
                *cell = InMaze(MazeCell::new(
                    direction == West || last_direction == Some(East),
                    direction == North || last_direction == Some(South),
                ));
                curr_idx = self.adjacent_index(curr_idx, direction).unwrap();
                last_direction = Some(direction);
            }

            // Remove unused walk cells. This does not need to be done except when visualizing the
            // maze generation. Keep it in here anyway because visualization is helpful (and cool)
            // and it has minimal impact on performance.
            for idx in walk_indexes.iter() {
                if let cell @ Walk(_) = self.cell_mut(*idx) {
                    *cell = Empty;
                }
            }
        }

        println!("{}", self);

        let mut maze = Maze::new(self.width as u32, self.height as u32);
        for (idx, cell) in self.cells.iter().enumerate() {
            match cell {
                InMaze(maze_cell) => *maze.cell_mut(idx) = *maze_cell,
                _ => unreachable!(),
            }
        }
        maze
    }

    fn choose_walk_start(&mut self) -> Option<usize> {
        let mut candidate = self.unvisited_candidates.pop();
        while let Some(idx) = candidate {
            // Do not check explicitly for `Empty` so that walk paths do not have to be cleared
            // between walks. Walk paths only need to be cleared when visualizing the maze as it is
            // generated.
            if !matches!(self.cell(idx), Cell::InMaze { .. }) {
                return candidate;
            }
            candidate = self.unvisited_candidates.pop();
        }
        None
    }

    fn choose_random_adjacent<R: Rng>(
        &self,
        from_idx: usize,
        directions: &mut [Direction],
        rng: &mut R,
    ) -> (Direction, usize) {
        directions.shuffle(rng);
        let mut dir_idx = 0;
        let mut direction = directions[dir_idx];
        let mut adjacent = self.adjacent_index(from_idx, direction);
        while adjacent.is_none() && dir_idx < directions.len() {
            dir_idx += 1;
            direction = directions[dir_idx];
            adjacent = self.adjacent_index(from_idx, direction);
        }
        // There is guaranteed to be a valid adjacent cell
        (direction, adjacent.unwrap())
    }
}

impl Display for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..(self.height) {
            for col in 0..self.width {
                f.write_char('-')?;
                let idx = row * self.width + col;
                match self.cell(idx) {
                    Cell::InMaze(mc) if mc.north_open() => f.write_char(' ')?,
                    _ => f.write_char('-')?,
                }
            }
            f.write_char('-')?;
            f.write_char('\n')?;
            for col in 0..self.width {
                let idx = row * self.width + col;
                match self.cell(idx) {
                    Cell::InMaze(mc) if mc.west_open() => f.write_char(' ')?,
                    _ => f.write_char('|')?,
                }
                match self.cell(idx) {
                    Cell::Empty => f.write_char('X')?,
                    Cell::InMaze { .. } => f.write_char(' ')?,
                    Cell::Walk(Direction::North) => f.write_char('^')?,
                    Cell::Walk(Direction::East) => f.write_char('>')?,
                    Cell::Walk(Direction::South) => f.write_char('v')?,
                    Cell::Walk(Direction::West) => f.write_char('<')?,
                }
            }

            f.write_char('|')?;
            f.write_char('\n')?;
        }
        for _ in 0..self.width {
            f.write_char('-')?;
            f.write_char('-')?;
        }
        f.write_char('-')?;
        f.write_char('\n')?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Cell {
    Empty,
    /// Maze cells "own" their west and north walls
    InMaze(MazeCell),
    Walk(Direction),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}
