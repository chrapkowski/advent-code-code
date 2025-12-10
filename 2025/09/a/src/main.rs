use std::{
    error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    path::Path,
    str::FromStr,
};

struct Point {
    x: u64,
    y: u64,
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl Point {}

#[derive(Debug)]
enum ParsePointError {
    WrongFieldCount,
    InvalidNumber(ParseIntError),
}

impl Display for ParsePointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongFieldCount => write!(f, "Wrong field count"),
            Self::InvalidNumber(e) => write!(f, "Invalid number: {e}"),
        }
    }
}

impl error::Error for ParsePointError {}

impl FromStr for Point {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(",").collect();

        if parts.len() != 2 {
            return Err(ParsePointError::WrongFieldCount);
        }

        let coordinates: Vec<u64> = parts
            .into_iter()
            .map(|it| it.parse::<u64>().map_err(ParsePointError::InvalidNumber))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Point {
            x: coordinates[0],
            y: coordinates[1],
        })
    }
}

fn read_points(path: &Path) -> io::Result<Vec<Point>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            line?
                .parse::<Point>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        })
        .collect::<Result<Vec<Point>, _>>()
}

fn main() -> io::Result<()> {
    let mut points = read_points(Path::new("../input"))?;
    points.sort_by_key(|p| (p.y, p.x));
    let points = &points;

    let areas = (0..points.len().saturating_sub(1))
        .flat_map(|i| {
            (i + 1..points.len()).map(move |j| {
                let p = &points[i];
                let q = &points[j];

                if q.x < p.x {
                    return 0;
                }

                let dx = q.x - p.x + 1;
                let dy = q.y - p.y + 1;

                dx * dy
            })
        })
        .max();

    println!("{}", areas.unwrap());

    Ok(())
}
