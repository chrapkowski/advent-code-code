use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::Path,
    sync::LazyLock,
};

use regex::Regex;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Grid {
    data: Vec<char>,
    width: usize,
    height: usize,
}

impl Grid {
    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        self.width * y + x
    }

    fn new(width: usize, height: usize) -> Self {
        let data = vec![' '; width * height];
        Self { width, height, data }
    }

    fn get(&self, x: usize, y: usize) -> Option<char> {
        if x >= self.width {
            return None;
        }

        if y >= self.height {
            return None;
        }

        Some(self.data[self.index(x, y)])
    }

    fn set(&mut self, x: usize, y: usize, c: char) {
        if x >= self.width {
            return;
        }

        if y >= self.height {
            return;
        }

        let index = self.index(x, y);
        self.data[index] = c;
    }

    fn filled_cells(&self) -> Vec<(usize, usize)> {
        let mut filled = Vec::new();

        for (idx, c) in self.data.iter().enumerate() {
            if *c != ' ' {
                filled.push((idx % self.width, idx / self.width));
            }
        }

        filled
    }
}

struct Region {
    width: usize,
    height: usize,
    shape_quantities: Vec<usize>,
}

struct Solver<'a> {
    region: &'a Region,
    transforms: &'a Transformer,
}

impl<'a> Solver<'_> {
    fn can_fit(&'a self) -> bool {
        let mut grid = Grid::new(self.region.width, self.region.height);
        let mut remaining = self.region.shape_quantities.clone();

        let area = self.region.width * self.region.height;
        let total_filled: usize = self
            .region
            .shape_quantities
            .iter()
            .enumerate()
            .map(|(shape_id, count)| count * self.transforms.filled_count(shape_id))
            .sum();

        if total_filled > area {
            return false;
        }

        let empty_budget = area - total_filled;

        self.can_fit_grid(&mut grid, &mut remaining, empty_budget, 0)
    }

    fn can_fit_grid(&self, grid: &mut Grid, remaining: &mut [usize], remaining_empty: usize, depth: usize) -> bool {
        let total_remaining: usize = remaining.iter().sum();

        if total_remaining == 0 {
            return true;
        }

        let Some((anchor_x, anchor_y)) = self.first_empty_cell(grid) else {
            return false;
        };

        for shape_id in 0..remaining.len() {
            if remaining[shape_id] == 0 {
                continue;
            }

            let variants = self.transforms.variants(shape_id);

            for variant in variants {
                for &(dx, dy) in &variant.filled {
                    if anchor_x < dx || anchor_y < dy {
                        continue;
                    }

                    let x = anchor_x - dx;
                    let y = anchor_y - dy;

                    if x + variant.grid.width > self.region.width || y + variant.grid.height > self.region.height {
                        continue;
                    }

                    if self.can_place_at(grid, variant, x, y) {
                        self.place_at(grid, variant, x, y, '#');
                        remaining[shape_id] -= 1;

                        if self.can_fit_grid(grid, remaining, remaining_empty, depth + 1) {
                            return true;
                        }

                        remaining[shape_id] += 1;
                        self.place_at(grid, variant, x, y, ' ');
                    }
                }
            }
        }

        if remaining_empty > 0 {
            grid.set(anchor_x, anchor_y, '.');

            if self.can_fit_grid(grid, remaining, remaining_empty - 1, depth + 1) {
                return true;
            }

            grid.set(anchor_x, anchor_y, ' ');
        }

        false
    }

    fn first_empty_cell(&self, grid: &Grid) -> Option<(usize, usize)> {
        for (idx, c) in grid.data.iter().enumerate() {
            if *c == ' ' {
                return Some((idx % grid.width, idx / grid.width));
            }
        }

        None
    }

    fn can_place_at(&self, grid: &Grid, variant: &Variant, x: usize, y: usize) -> bool {
        for &(dx, dy) in &variant.filled {
            let gx = x + dx;
            let gy = y + dy;

            if grid.get(gx, gy) != Some(' ') {
                return false;
            }
        }

        true
    }

    fn place_at(&self, grid: &mut Grid, variant: &Variant, x: usize, y: usize, c: char) {
        for &(dx, dy) in &variant.filled {
            grid.set(x + dx, y + dy, c);
        }
    }
}

struct Transformation {
    swap_xy: bool,
    invert_x: bool,
    invert_y: bool,
}

impl Transformation {
    fn apply(&self, grid: &Grid) -> Grid {
        let old_w = grid.width;
        let old_h = grid.height;

        let new_w = if self.swap_xy { old_h } else { old_w };
        let new_h = if self.swap_xy { old_w } else { old_h };

        let mut result = vec![' '; new_w * new_h];

        for y in 0..old_h {
            for x in 0..old_w {
                let mut cx = x;
                let mut cy = y;

                if self.invert_x {
                    cx = old_w - 1 - cx;
                }

                if self.invert_y {
                    cy = old_h - 1 - cy;
                }

                let (final_x, final_y) = if self.swap_xy { (cy, cx) } else { (cx, cy) };

                let old_idx = y * old_w + x;
                let new_idx = final_y * new_w + final_x;

                result[new_idx] = grid.data[old_idx];
            }
        }

        Grid {
            data: result,
            width: new_w,
            height: new_h,
        }
    }
}

const TRANSFORMATIONS: [Transformation; 8] = [
    Transformation {
        swap_xy: false,
        invert_x: false,
        invert_y: false,
    }, // R0
    Transformation {
        swap_xy: true,
        invert_x: false,
        invert_y: true,
    }, // R90
    Transformation {
        swap_xy: false,
        invert_x: true,
        invert_y: true,
    }, // R180
    Transformation {
        swap_xy: true,
        invert_x: true,
        invert_y: false,
    }, // R270
    Transformation {
        swap_xy: false,
        invert_x: true,
        invert_y: false,
    }, // HFlip
    Transformation {
        swap_xy: true,
        invert_x: true,
        invert_y: true,
    }, // R90H
    Transformation {
        swap_xy: false,
        invert_x: false,
        invert_y: true,
    }, // VFlip (R180H)
    Transformation {
        swap_xy: true,
        invert_x: false,
        invert_y: false,
    }, // R270H
];

struct Variant {
    grid: Grid,
    filled: Vec<(usize, usize)>,
}

struct Transformer {
    variants: Vec<Vec<Variant>>,
    filled_counts: Vec<usize>,
}

impl Transformer {
    fn new(shapes: &[Grid]) -> Self {
        let mut variants = Vec::with_capacity(shapes.len());
        let mut filled_counts = Vec::with_capacity(shapes.len());

        for shape in shapes {
            filled_counts.push(shape.filled_cells().len());
            let mut seen = HashSet::new();
            let mut list = Vec::new();

            for transformation in &TRANSFORMATIONS {
                let transformed = transformation.apply(shape);

                if seen.insert(transformed.clone()) {
                    let filled = transformed.filled_cells();
                    list.push(Variant {
                        grid: transformed,
                        filled,
                    });
                }
            }

            variants.push(list);
        }

        Self {
            variants,
            filled_counts,
        }
    }

    fn variants(&self, shape_id: usize) -> &[Variant] {
        &self.variants[shape_id]
    }

    fn filled_count(&self, shape_id: usize) -> usize {
        self.filled_counts[shape_id]
    }
}

struct Input {
    shapes: Vec<Grid>,
    regions: Vec<Region>,
}

static SHAPE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+):$").unwrap());
static REGION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)x(\d+):(( \d+)+)$").unwrap());

impl Input {
    fn parse_grid(lines: &mut Lines<BufReader<File>>) -> io::Result<Grid> {
        let mut rows = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                break;
            }

            rows.push(trimmed.to_string());
        }

        if rows.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty grid"));
        }

        let width = rows[0].len();

        if rows.iter().any(|r| r.len() != width) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "All grid rows must have the same length",
            ));
        }

        let height = rows.len();

        let data: Vec<char> = rows
            .into_iter()
            .flat_map(|s| s.into_bytes())
            .map(|b| match b as char {
                '.' => ' ',
                other => other,
            })
            .collect();

        Ok(Grid { data, width, height })
    }

    fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut shapes: Vec<Grid> = Vec::new();
        let mut regions: Vec<Region> = Vec::new();

        while let Some(line) = lines.next() {
            let line = line?;

            if let Some(captures) = REGION_REGEX.captures(&line) {
                let width: usize = captures[1].parse().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid region width '{}': {}", &captures[1], e),
                    )
                })?;

                let height: usize = captures[2].parse().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid region height '{}': {}", &captures[1], e),
                    )
                })?;

                let shape_quantities: Vec<usize> = captures[3]
                    .split_ascii_whitespace()
                    .map(|s| s.parse())
                    .collect::<Result<_, _>>()
                    .map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid quantity in region: {}", e))
                    })?;

                regions.push(Region {
                    width,
                    height,
                    shape_quantities,
                });
            } else {
                let Some(captures) = SHAPE_REGEX.captures(&line) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unexpected line format: '{}'", line),
                    ));
                };

                let id: usize = captures[1].parse().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid shape ID '{}': {}", &captures[1], e),
                    )
                })?;

                let expected_id = shapes.len();

                if expected_id != id {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Shape ID mismatch: expected {}, got {} (shapes parsed so far)",
                            expected_id, id
                        ),
                    ));
                }

                let grid = Input::parse_grid(&mut lines)?;
                shapes.push(grid);
            }
        }

        return Ok(Input { shapes, regions });
    }

    fn count(&self) -> u32 {
        let mut count = 0;
        let transformer = Transformer::new(&self.shapes);

        for region in &self.regions {
            let solver = Solver {
                region,
                transforms: &transformer,
            };

            if solver.can_fit() {
                count += 1;
            }
        }

        count
    }
}

fn main() -> io::Result<()> {
    let input = Input::load(Path::new("../input"))?;
    let count = input.count();

    println!("{count}");

    Ok(())
}
