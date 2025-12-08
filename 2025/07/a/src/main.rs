use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

struct Grid {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

impl Grid {
    fn get(&self, i: usize, j: usize) -> Option<char> {
        self.cells.get(self.width * j + i).copied()
    }

    fn set(&mut self, i: usize, j: usize, c: char) {
        if let Some(it) = self.cells.get_mut(self.width * j + i) {
            *it = c;
        }
    }
}

fn read_char_grid(reader: &mut BufReader<File>) -> io::Result<Grid> {
    let mut width = 0_usize;
    let mut height = 0_usize;
    let mut cells: Vec<char> = vec![];

    for line in reader.lines() {
        let chars: Vec<char> = line?.chars().collect();

        height += 1;
        cells.extend(&chars);

        if width == 0 {
            width = chars.len();
        }
    }

    Ok(Grid { width, height, cells })
}

fn main() -> io::Result<()> {
    let file = File::open(Path::new("../input"))?;
    let mut reader = BufReader::new(file);
    let mut grid = read_char_grid(&mut reader)?;
    let mut count = 0;

    for y in 0..grid.height - 1 {
        for x in 0..grid.width {
            let current= grid.get(x, y).unwrap();

            match current {
                'S' | '|' => {
                    let below = grid.get(x, y + 1).unwrap();

                    if below == '^' {
                        if x > 0 {
                            grid.set(x - 1, y + 1, '|');
                        }

                        if x < grid.width - 1 {
                            grid.set(x + 1, y + 1, '|');
                        }

                        count += 1;
                    } else {
                        grid.set(x, y + 1, '|');
                    }
                },
                _ => {}
            };
        }
    }

    println!("{count}");
    Ok(())
}
