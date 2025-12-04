use std::{
    fs::File,
    io::{self, BufRead},
    path::Path
};

struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn offset(&self, dx: isize, dy: isize) -> Option<Self> {
        let x = self.x as isize + dx;
        let y = self.y as isize + dy;

        if x < 0 || y < 0 {
            return None;
        }

        Some(Position { x: x as usize, y: y as usize })
    }
}

struct Shelf {
    data: Vec<char>,
    width: usize,
    height: usize
}

impl Shelf {
    fn new(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let mut width= 0;
        let mut data: Vec<char> = vec![];
        let mut height = 0;

        for line in reader.lines() {
            let line = line?;

            if width == 0 {
                width = line.len();
            }

            for c in line.chars() {
                data.push(c);
            }

            height += 1;
        }

        Ok(Self { data, width, height })
    }

    fn get(&self, p: &Position) -> Option<&char> {
        if p.x >= self.width {
            return None;
        }

        self.data.get(p.y * self.width + p.x)
    }

    fn can_remove(&self, p: &Position) -> bool {
        let mut sum = 0;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue
                }

                if let Some(neighbour) = p.offset(dx, dy) {
                    if let Some('@') = self.get(&neighbour) {
                        sum += 1;

                        if sum == 4 {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }
}

fn main() -> io::Result<()> {
    let shelf = Shelf::new(Path::new("../input"))?;
    let mut count = 0;

    for y in 0..shelf.height {
        for x in 0..shelf.width {
            let p = Position { x, y };

            let Some('@') = shelf.get(&p) else {
                continue;
            };

            if shelf.can_remove(&p) {
                count += 1;
            }
        }
    }

    println!("{count}");

    Ok(())
}
