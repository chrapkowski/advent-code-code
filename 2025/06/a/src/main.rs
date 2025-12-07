use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

struct Input {
    width: usize,
    height: usize,
    numbers: Vec<u64>,
    operations: Vec<Operation>,
}

impl Input {
    fn get(&self, x: usize, y: usize) -> Option<u64> {
        self.numbers.get(y * self.width + x).copied()
    }
}

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
    fn apply(&self, accumulator: &mut u64, value: u64) {
        match self {
            Operation::Add => *accumulator += value,
            Operation::Multiply => *accumulator *= value,
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

fn parse_line<T>(line: &str) -> io::Result<Vec<T>>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    line.split_ascii_whitespace()
        .map(|it| it.parse::<T>())
        .collect::<Result<_, _>>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_input(path: &Path) -> io::Result<Input> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut width = 0_usize;
    let mut height = 0_usize;
    let mut numbers: Vec<u64> = vec![];
    let mut operations: Vec<Operation> = vec![];

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with(|c: char| c.is_ascii_digit()) {
            height += 1;

            let mut row = parse_line(line)?;
            numbers.append(&mut row);

            if width == 0 {
                width = numbers.len();
            }

            continue;
        }

        operations = parse_line(line)?;
    }

    Ok(Input {
        width,
        height,
        numbers,
        operations,
    })
}

fn main() -> io::Result<()> {
    let input = read_input(Path::new("../input"))?;
    let mut sum = 0_u64;

    for i in 0..input.width {
        let mut result = input.get(i, 0).unwrap();
        let operation = input.operations.get(i).unwrap();

        for j in 1..input.height {
            operation.apply(&mut result, input.get(i, j).unwrap());
        }

        sum += result;
    }

    println!("{sum}");

    Ok(())
}
