use std::{
    error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    path::Path,
    str::FromStr,
    usize,
};

use regex::Regex;

type Buttons = Vec<u64>;

struct Machine {
    _joltages: Vec<u64>,
    buttons: Buttons,
    width: usize,
    target: u64,
}

struct ButtonsFmt<'a>(&'a [u64], usize);

fn reverse_bits(number: u64, width: usize) -> u64 {
    let mut reverse = 0;
    let mut number = number;

    for _ in 0..width {
        reverse <<= 1;
        reverse |= number & 1;
        number >>= 1;
    }

    reverse
}

impl<'a> Display for ButtonsFmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, button) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }

            write!(f, "(")?;

            let mut button = reverse_bits(*button, self.1);
            let mut j = 0;
            let mut first = true;

            while button != 0 {
                if button & 1 == 1 {
                    if !first {
                        write!(f, ",")?;
                    }

                    write!(f, "{j}")?;

                    first = false;
                }

                j += 1;
                button >>= 1;
            }

            write!(f, ")")?;
        }

        Ok(())
    }
}

fn fmt_indicators(indicators: u64, width: usize) -> String {
    format!("[{:0width$b}]", indicators, width = width)
        .replace('0', ".")
        .replace('1', "#")
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", fmt_indicators(self.target, self.width))?;

        ButtonsFmt(&self.buttons, self.width).fmt(f)?;

        let joltages = self
            ._joltages
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, " {{{joltages}}}")
    }
}

#[derive(Debug)]
enum ParseMachineError {
    InvalidSyntax,
    InvalidNumber(ParseIntError),
}

impl Display for ParseMachineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSyntax => write!(f, "Invalid syntax"),
            Self::InvalidNumber(e) => write!(f, "Invalid number: {e}"),
        }
    }
}

impl error::Error for ParseMachineError {}

impl FromStr for Machine {
    type Err = ParseMachineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let outer_re = Regex::new(r"\[(?<target>[\.#]{1,64})\] (?<buttons>.+?) (?P<joltages>\{([\d,]+)\})").unwrap();
        let inner_re = Regex::new(r"\((?<indices>[\d,]+)\)").unwrap();

        let captures = outer_re.captures(s).ok_or(ParseMachineError::InvalidSyntax)?;

        let target = &captures["target"];
        let width = target.len();
        let target = target
            .chars()
            .fold(0u64, |acc, c| (acc << 1) | (if c == '#' { 1 } else { 0 }));

        let buttons: Result<Vec<u64>, _> = inner_re
            .captures_iter(&captures["buttons"])
            .map(|captures| {
                captures["indices"]
                    .split(',')
                    .map(|nummber| nummber.parse::<u64>().map_err(ParseMachineError::InvalidNumber))
                    .try_fold(0_u64, |acc, num| num.map(|num| acc | 1 << num))
                    .map(|number| reverse_bits(number, width))
            })
            .collect();

        let joltages: Vec<u64> = captures["joltages"]
            .trim_matches(|c| c == '{' || c == '}')
            .split(',')
            .map(|number| number.parse::<u64>().map_err(ParseMachineError::InvalidNumber))
            .collect::<Result<_, _>>()?;

        Ok(Machine {
            target,
            width,
            buttons: buttons?,
            _joltages: joltages,
        })
    }
}

fn read_machines(path: &Path) -> io::Result<Vec<Machine>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            line?
                .parse::<Machine>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        })
        .collect::<Result<Vec<Machine>, _>>()
}

struct Permutations {
    n: usize,
    k: usize,
    stack: Vec<(usize, usize, u64)>,
}

impl Permutations {
    fn new(n: usize) -> Self {
        Self {
            n,
            k: 0,
            stack: vec![(n, 0, 0)],
        }
    }
}

impl Iterator for Permutations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stack.is_empty() {
                if self.k == self.n {
                    return None;
                }

                self.k += 1;
                self.stack.push((self.n, self.k, 0));
            }

            let (n, k, v) = self.stack.pop().unwrap();

            if k == 0 {
                return Some(permutation_to_indices(v << n));
            }

            if n > k {
                self.stack.push((n - 1, k, v << 1));
            }

            self.stack.push((n - 1, k - 1, (v << 1) | 1));
        }
    }
}

fn permutation_to_indices(permutation: u64) -> Vec<usize> {
    let mut result = vec![];
    let mut current = permutation;
    let mut index = 0;

    while current != 0 {
        if current & 1 == 1 {
            result.push(index);
        }

        index += 1;
        current = current >> 1;
    }

    return result;
}

fn main() -> io::Result<()> {
    let machines = read_machines(Path::new("../input"))?;
    let mut sum = 0;

    for machine in machines {
        let n = machine.buttons.len();

        for indices in Permutations::new(n) {
            let buttons = indices.into_iter().map(|i| machine.buttons[i]).collect::<Vec<_>>();
            let outcome = buttons.iter().fold(0, |acc, button| acc ^ button);
            if outcome == machine.target {
                sum += buttons.len();
                break;
            }
        }
    }

    println!("{sum}");

    Ok(())
}
