use std::{
    char,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

struct Grid<T: Copy> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T: Copy> Grid<T> {
    fn new(width: usize, height: usize, init: T) -> Self {
        let cells = vec![init; width * height];

        Self { width, height, cells }
    }

    fn get(&self, i: usize, j: usize) -> Option<T> {
        self.cells.get(self.width * j + i).copied()
    }

    fn set(&mut self, i: usize, j: usize, c: T) {
        if let Some(it) = self.cells.get_mut(self.width * j + i) {
            *it = c;
        }
    }
}

fn read_char_grid(reader: &mut BufReader<File>) -> io::Result<Grid<char>> {
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
    let char_grid = read_char_grid(&mut reader)?;
    let mut number_grid = Grid::new(char_grid.width, char_grid.height, 0_u64);

    for y in 0..char_grid.height {
        for x in 0..char_grid.width {
            let current = char_grid.get(x, y).unwrap();

            match current {
                'S' => {
                    number_grid.set(x, y, 1);
                }
                '^' => {
                    let above = number_grid.get(x, y - 1).unwrap();

                    if x > 0 {
                        let before = number_grid.get(x - 1, y).unwrap();
                        number_grid.set(x - 1, y, before + above);
                    }

                    if x < char_grid.width - 1 {
                        let before = number_grid.get(x + 1, y).unwrap();
                        number_grid.set(x + 1, y, before + above);
                    }
                }
                _ => {
                    if y == 0 {
                        continue;
                    }

                    let above = number_grid.get(x, y - 1).unwrap();
                    let before = number_grid.get(x, y).unwrap();

                    number_grid.set(x, y, before + above);
                }
            };
        }
    }

    let count = (0..char_grid.width)
        .map(|i| number_grid.get(i, char_grid.height - 1).unwrap())
        .sum::<u64>();

    println!("{count}");
    Ok(())
}
