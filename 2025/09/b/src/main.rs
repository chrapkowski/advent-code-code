use std::{
    cmp::Ordering,
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
        let parts: Vec<&str> = s.split(',').collect();

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

struct Edge<'a> {
    s: &'a Point,
    e: &'a Point,
}

fn compare_horizontal(a: &Edge, b: &Edge) -> Ordering {
    let ya = a.s.y;
    let yb = b.s.y;

    if ya != yb {
        return ya.cmp(&yb);
    }

    let ax0 = a.s.x.min(a.e.x);
    let bx0 = b.s.x.min(b.e.x);
    ax0.cmp(&bx0)
}

fn compare_vertical(a: &Edge, b: &Edge) -> Ordering {
    let xa = a.s.x;
    let xb = b.s.x;

    if xa != xb {
        return xa.cmp(&xb);
    }

    let ay0 = a.s.y.min(a.e.y);
    let by0 = b.s.y.min(b.e.y);
    ay0.cmp(&by0)
}

fn lower_bound<T, F>(slice: &[T], mut f: F) -> usize
where
    F: FnMut(&T) -> bool,
{
    let mut left = 0;
    let mut right = slice.len();
    while left < right {
        let mid = (left + right) / 2;
        if f(&slice[mid]) {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

fn overlaps(a0: u64, a1: u64, b0: u64, b1: u64) -> bool {
    !(a1 <= b0 || b1 <= a0)
}

struct Solver<'a> {
    points: &'a [Point],
    edges: Vec<Edge<'a>>,
    h_edges: Vec<usize>,
    v_edges: Vec<usize>,
    left_edges: Vec<usize>,
    right_edges: Vec<usize>,
    up_edges: Vec<usize>,
    down_edges: Vec<usize>,
}

impl<'a> Solver<'a> {
    fn new(points: &'a [Point]) -> Self {
        let edges = (0..points.len())
            .map(|i| Edge {
                s: &points[i],
                e: &points[(i + 1) % points.len()],
            })
            .collect::<Vec<_>>();

        let mut h_edges = (0..edges.len())
            .filter(|i| edges[*i].s.y == edges[*i].e.y)
            .collect::<Vec<usize>>();
        let mut v_edges = (0..edges.len())
            .filter(|i| edges[*i].s.x == edges[*i].e.x)
            .collect::<Vec<usize>>();

        h_edges.sort_by(|a, b| compare_horizontal(&edges[*a], &edges[*b]));
        v_edges.sort_by(|a, b| compare_vertical(&edges[*a], &edges[*b]));

        let mut left_edges = h_edges
            .iter()
            .copied()
            .filter(|i| edges[*i].s.x > edges[*i].e.x)
            .collect::<Vec<usize>>();
        let mut right_edges = h_edges
            .iter()
            .copied()
            .filter(|i| edges[*i].s.x < edges[*i].e.x)
            .collect::<Vec<usize>>();
        let mut up_edges = v_edges
            .iter()
            .copied()
            .filter(|i| edges[*i].s.y > edges[*i].e.y)
            .collect::<Vec<usize>>();
        let mut down_edges = v_edges
            .iter()
            .copied()
            .filter(|i| edges[*i].s.y < edges[*i].e.y)
            .collect::<Vec<usize>>();

        left_edges.sort_by(|a, b| compare_horizontal(&edges[*a], &edges[*b]));
        right_edges.sort_by(|a, b| compare_horizontal(&edges[*a], &edges[*b]));
        up_edges.sort_by(|a, b| compare_vertical(&edges[*a], &edges[*b]));
        down_edges.sort_by(|a, b| compare_vertical(&edges[*a], &edges[*b]));

        Solver {
            points,
            edges,
            h_edges,
            v_edges,
            left_edges,
            right_edges,
            up_edges,
            down_edges,
        }
    }

    fn horizontal_cross(&self, x0: u64, x1: u64, y0: u64, y1: u64) -> bool {
        let start = lower_bound(&self.h_edges, |idx| self.edges[*idx].s.y <= y0);
        for idx in &self.h_edges[start..] {
            let e = &self.edges[*idx];
            let y = e.s.y;

            if y >= y1 {
                break;
            }

            let ex0 = e.s.x.min(e.e.x);
            let ex1 = e.s.x.max(e.e.x);

            if ex1 > x0 && ex0 < x1 {
                return true;
            }
        }
        false
    }

    fn vertical_cross(&self, x0: u64, x1: u64, y0: u64, y1: u64) -> bool {
        let start = lower_bound(&self.v_edges, |idx| self.edges[*idx].s.x <= x0);
        for idx in &self.v_edges[start..] {
            let e = &self.edges[*idx];
            let x = e.s.x;

            if x >= x1 {
                break;
            }

            let ey0 = e.s.y.min(e.e.y);
            let ey1 = e.s.y.max(e.e.y);

            if ey1 > y0 && ey0 < y1 {
                return true;
            }
        }
        false
    }

    fn horizontal_border_conflict(&self, indices: &[usize], x0: u64, x1: u64, y: u64) -> bool {
        let start = lower_bound(indices, |idx| self.edges[*idx].s.y < y);
        for idx in &indices[start..] {
            let e = &self.edges[*idx];
            let ey = e.s.y;

            if ey > y {
                break;
            }

            let ex0 = e.s.x.min(e.e.x);
            let ex1 = e.s.x.max(e.e.x);

            if overlaps(x0, x1, ex0, ex1) {
                return true;
            }
        }
        false
    }

    fn vertical_border_conflict(&self, indices: &[usize], y0: u64, y1: u64, x: u64) -> bool {
        let start = lower_bound(indices, |idx| self.edges[*idx].s.x < x);
        for idx in &indices[start..] {
            let e = &self.edges[*idx];
            let ex = e.s.x;

            if ex > x {
                break;
            }

            let ey0 = e.s.y.min(e.e.y);
            let ey1 = e.s.y.max(e.e.y);

            if overlaps(y0, y1, ey0, ey1) {
                return true;
            }
        }
        false
    }

    fn solve(&self) -> u64 {
        (0..self.points.len().saturating_sub(1))
            .flat_map(|i| (i + 1..self.points.len()).map(move |j| (i, j)))
            .filter(|v| v.0 != v.1)
            .map(|(i, j)| {
                let p = &self.points[i];
                let q = &self.points[j];

                let x0 = p.x.min(q.x);
                let x1 = p.x.max(q.x);
                let y0 = p.y.min(q.y);
                let y1 = p.y.max(q.y);

                let dx = x1 - x0 + 1;
                let dy = y1 - y0 + 1;
                let area = dx * dy;

                if self.horizontal_cross(x0, x1, y0, y1) || self.vertical_cross(x0, x1, y0, y1) {
                    return 0;
                }

                let top_conflict =
                    self.horizontal_border_conflict(&self.left_edges, x0, x1, y0);
                let bottom_conflict =
                    self.horizontal_border_conflict(&self.right_edges, x0, x1, y1);
                let left_conflict =
                    self.vertical_border_conflict(&self.down_edges, y0, y1, x0);
                let right_conflict =
                    self.vertical_border_conflict(&self.up_edges, y0, y1, x1);

                if top_conflict || bottom_conflict || left_conflict || right_conflict {
                    return 0;
                }

                area
            })
            .max()
            .unwrap()
    }
}

fn main() -> io::Result<()> {
    let points = read_points(Path::new("../input"))?;
    let solver = Solver::new(&points);
    let area = solver.solve();
    println!("{area}");
    Ok(())
}
