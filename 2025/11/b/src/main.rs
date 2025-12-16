use std::{
    collections::HashMap,
    error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

struct Server {
    name: String,
    connections: Vec<String>,
}

#[derive(Debug)]
enum ParseServerError {
    InvalidLineFormat(String),
}

impl Display for ParseServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseServerError::InvalidLineFormat(found) => write!(f, "line format is invalid, got '{}'", found),
        }
    }
}

impl error::Error for ParseServerError {}

impl FromStr for Server {
    type Err = ParseServerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let outer = s.splitn(2, ":").collect::<Vec<_>>();

        if outer.len() != 2 {
            return Result::Err(ParseServerError::InvalidLineFormat(s.to_owned()));
        }

        let name = outer[0].to_owned();
        let connections = outer[1]
            .trim()
            .split(" ")
            .map(|c| c.to_owned())
            .collect::<Vec<String>>();

        if connections.len() == 0 {
            return Result::Err(ParseServerError::InvalidLineFormat(s.to_owned()));
        }

        Ok(Self { name, connections })
    }
}

struct Input {
    servers: HashMap<String, Vec<String>>,
}

impl Input {
    fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        reader
            .lines()
            .map(|line| {
                line?
                    .parse::<Server>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
            })
            .collect::<Result<Vec<Server>, _>>()
            .map(|servers| Input {
                servers: servers.into_iter().map(|s| (s.name, s.connections)).collect(),
            })
    }
}

impl Input {
    fn count_paths(self: &Input, parent: &str, mask: i32, cache: &mut HashMap<(String, i32), u64>) -> u64 {
        let key = (parent.to_owned(), mask);

        if let Some(count) = cache.get(&key) {
            return *count;
        }

        let mut count = 0;

        for child in &self.servers[parent] {
            if child == TERMINAL {
                if mask == 3 {
                    count += 1;
                }
                continue;
            }

            let mut mask = mask;
            if child == "fft" {
                mask |= 1;
            } else if child == "dac" {
                mask |= 2;
            }

            count += self.count_paths(child, mask, cache);
        }

        cache.insert(key, count);

        count
    }
}

const START: &str = "svr";
const TERMINAL: &str = "out";

fn main() -> io::Result<()> {
    let input = Input::load(Path::new("../input"))?;
    let count = input.count_paths(START, 0, &mut HashMap::new());

    println!("{count}");

    Ok(())
}
