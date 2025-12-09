use std::{
    error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    path::Path,
    str::FromStr,
};

#[derive(Clone)]
struct Point {
    x: i64,
    y: i64,
    z: i64,
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl Point {
    fn distance_to(&self, other: &Point) -> u64 {
        ((self.x - other.x).pow(2) + (self.y - other.y).pow(2) + (self.z - other.z).pow(2)) as u64
    }
}

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

        if parts.len() != 3 {
            return Err(ParsePointError::WrongFieldCount);
        }

        let coordinates: Vec<i64> = parts
            .into_iter()
            .map(|it| it.parse::<i64>().map_err(ParsePointError::InvalidNumber))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Point {
            x: coordinates[0],
            y: coordinates[1],
            z: coordinates[2],
        })
    }
}

trait DisjointSetUnion {
    fn find(&mut self, i: usize) -> usize;
    fn union(&mut self, i: usize, j: usize);
}

impl DisjointSetUnion for Vec<usize> {
    fn find(&mut self, i: usize) -> usize {
        if self[i] != i {
            self[i] = self.find(self[i])
        }

        self[i]
    }

    fn union(&mut self, i: usize, j: usize) {
        let p = self.find(i);
        let q = self.find(j);

        if p == q {
            return;
        }

        self[q] = p;
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

fn calculate_pairwise_distances(points: &[Point]) -> Vec<(usize, usize, u64)> {
    let pair_count = points.len().saturating_mul(points.len().saturating_sub(1)) / 2;
    let mut distances: Vec<(usize, usize, u64)> = Vec::with_capacity(pair_count);

    for i in 0..points.len().saturating_sub(1) {
        for j in i + 1..points.len() {
            distances.push((i, j, points[i].distance_to(&points[j])));
        }
    }

    distances.sort_by_key(|p| p.2);
    distances
}

fn find_final_connection(points: &[Point], distances: &[(usize, usize, u64)]) -> Option<(Point, Point)> {
    let mut connections: Vec<_> = (0..points.len()).collect();
    let mut remaining = points.len();

    for (i, j, _) in distances.iter() {
        if connections.find(*i) == connections.find(*j) {
            continue;
        }

        connections.union(*i, *j);
        remaining -= 1;

        if remaining == 1 {
            let p = &points[*i];
            let q = &points[*j];
            return Some((p.to_owned(), q.to_owned()));
        }
    }

    None
}

fn main() -> io::Result<()> {
    let points = read_points(Path::new("../input"))?;
    let distances = calculate_pairwise_distances(&points);

    if let Some((p, q)) = find_final_connection(&points, &distances) {
        println!("{}", p.x * q.x);
    }

    Ok(())
}
