use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

enum Operation {
    Add,
    Multiply,
}

#[derive(Debug)]
struct ParseOperationError(String);

impl fmt::Display for ParseOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseOperationError {}

impl Operation {
    fn apply(&self, a: u64, b: u64) -> u64 {
        match self {
            Operation::Add => a + b,
            Operation::Multiply => a * b,
        }
    }
}

impl FromStr for Operation {
    type Err = ParseOperationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(Operation::Multiply),
            "+" => Ok(Operation::Add),
            _ => Err(ParseOperationError(format!("Unsupported operation {s}"))),
        }
    }
}

struct Grid {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

impl Grid {
    fn get(&self, i: usize, j: usize) -> Option<char> {
        self.cells.get(self.width * j + i).copied()
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
    let char_grid = read_char_grid(&mut reader)?;
    let mut operation = Operation::Add;
    let mut stack = vec![];

    for i in 0..char_grid.width {
        let mut text = String::with_capacity(char_grid.height - 1);

        for j in 0..char_grid.height - 1 {
            let c = char_grid.get(i, j).unwrap();

            if c.is_ascii_digit() {
                text.push(c);
            }
        }

        if text.is_empty() {
            continue;
        }

        let c = char_grid.get(i, char_grid.height - 1).unwrap();
        let number: u64 = text.parse().unwrap();

        if !c.is_whitespace() {
            operation = c.to_string().parse().unwrap();
            stack.push(number);
            continue;
        }

        let result = operation.apply(stack.pop().unwrap(), number);
        stack.push(result);
    }

    let mut sum = 0;
    for number in &stack {
        sum += number;
    }

    println!("{sum}");

    Ok(())
}
