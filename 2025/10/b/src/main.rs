use std::{
    collections::HashMap, error, fmt::{self, Display, Formatter}, fs::File, io::{self, BufRead, BufReader}, iter, num::ParseIntError, path::Path, str::FromStr
};

use regex::Regex;

#[derive(Clone)]
struct State {
    joltages: Vec<u32>,
}

impl State {
    fn new(joltages: Vec<u32>) -> Self {
        State { joltages }
    }

    fn add(&self, rhs: &Self) -> Self {
        let mut joltages = vec![0; self.joltages.len()];

        for i in 0..self.joltages.len() {
            joltages[i] = self.joltages[i] + rhs.joltages[i];
        }

        Self { joltages }
    }

    fn mul(&self, rhs: u32) -> Self {
        let mut joltages = vec![0; self.joltages.len()];

        for i in 0..self.joltages.len() {
            joltages[i] = self.joltages[i] * rhs;
        }

        Self { joltages }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.joltages == other.joltages
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let joltages = self
            .joltages
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, " {{{joltages}}}")
    }
}

struct Machine {
    target: State,
    buttons: Vec<State>,
    width: usize,
    indicators: u64,
}

struct ButtonsFmt<'a>(&'a [State]);

impl<'a> Display for ButtonsFmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, state) in self.0.iter().enumerate() {
            let joltages = &state.joltages;

            if i > 0 {
                write!(f, " ")?;
            }

            write!(f, "(")?;
            let mut first = true;

            for j in 0..joltages.len() {
                if joltages[j] == 1 {
                    if !first {
                        write!(f, ",")?;
                    }

                    write!(f, "{j}")?;
                    first = false;
                }
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
        write!(f, "{} ", fmt_indicators(self.indicators, self.width))?;

        ButtonsFmt(&self.buttons).fmt(f)?;

        self.target.fmt(f)
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

        let buttons: Result<Vec<State>, _> = inner_re
            .captures_iter(&captures["buttons"])
            .map(|captures| {
                let mut joltages = vec![0_u32; width];

                captures["indices"].split(',').try_for_each(|number| {
                    let index = number.parse::<usize>().map_err(ParseMachineError::InvalidNumber)?;

                    joltages[index] = 1;
                    Ok(())
                })?;

                Ok(State::new(joltages))
            })
            .collect();

        let joltages: Vec<u32> = captures["joltages"]
            .trim_matches(|c| c == '{' || c == '}')
            .split(',')
            .map(|number| number.parse::<u32>().map_err(ParseMachineError::InvalidNumber))
            .collect::<Result<_, _>>()?;

        Ok(Machine {
            indicators: target,
            width,
            buttons: buttons?,
            target: State::new(joltages),
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

enum Stage {
    Initial,
    Continuation,
    Final,
}

struct Permutations {
    stack: Vec<(usize, u32, Box<dyn Iterator<Item = u32>>, Stage)>,
    current: Vec<u32>,
    maxes: Vec<u32>,
}

impl Permutations {
    fn new(maxes: Vec<u32>, count: u32) -> Self {
        let len = maxes.len();

        Self {
            stack: vec![(len, count, Box::new(iter::empty()), Stage::Initial)],
            current: Vec::with_capacity(len),
            maxes,
        }
    }
}

impl Iterator for Permutations {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (slots, remaining, mut iterator, stage) = self.stack.pop().unwrap();

            match stage {
                Stage::Initial => {
                    if slots == 0 {
                        if remaining == 0 {
                            return Some(self.current.clone());
                        }

                        continue;
                    }

                    let limit = self.maxes[self.current.len()].min(remaining);

                    self.stack
                        .push((slots, remaining, Box::new((0_u32..=limit).rev()), Stage::Continuation));
                }
                Stage::Continuation => {
                    if let Some(value) = iterator.next() {
                        self.current.push(value);
                        self.stack.push((slots, remaining, iterator, Stage::Final));
                        self.stack
                            .push((slots - 1, remaining - value, Box::new(iter::empty()), Stage::Initial));
                    }
                }
                Stage::Final => {
                    self.current.pop();
                    self.stack.push((slots, remaining, iterator, Stage::Continuation));
                }
            }

            if self.stack.is_empty() {
                return None;
            }
        }
    }
}

fn main() -> io::Result<()> {
    let machines = read_machines(Path::new("../input"))?;
    let mut sum = 0;

    for machine in &machines {
        println!("2. {machine}");

        let rhs = machine.target.joltages.clone();
        let coefficients = machine
            .buttons
            .iter()
            .map(|button| button.joltages.to_owned())
            .collect::<Vec<_>>();

        let (coefficients, rhs) = build_system_from_buttons(coefficients, rhs);
        let (coefficients, rhs) = simplify(coefficients, rhs);
        let (coefficients, rhs, count) = extract_trivial_solutions(coefficients, rhs);
        let (coefficients, rhs) = buttons_from_system(coefficients, rhs);

        sum += count;

        // ---- here was the mess before ----

        // Keep a copy of rhs as Vec<u32> for Solver
        let target_vec = rhs.clone();
        // Keep a copy of coefficients as Vec<Vec<u32>> for Solver
        let buttons_vec = coefficients.clone();

        // State versions for printing and for `current`
        let target = State { joltages: rhs };
        let buttons_states = buttons_vec
            .iter()
            .cloned()
            .map(|joltages| State { joltages })
            .collect::<Vec<_>>();

        println!("3. {}", ButtonsFmt(&buttons_states));
        println!("4. {}", target);

        let len = target.joltages.len();

        // build solver on raw vectors
        let solver = Solver::new(target_vec, buttons_vec);

        // button ids: 0..N
        let button_ids: Vec<usize> = (0..solver.buttons.len()).collect();

        let current = vec![0_u32; solver.len];
        let count = solver.solve().unwrap();

        println!("5. count = {count}");
        sum += count;
    }

    println!("6. {sum}");

    Ok(())
}

fn buttons_from_system(coefficients: Vec<Vec<u32>>, rhs: Vec<u32>) -> (Vec<Vec<u32>>, Vec<u32>) {
    let rows = coefficients.len();
    if rows == 0 {
        return (Vec::new(), rhs);
    }

    let cols = coefficients[0].len();
    let mut buttons = Vec::new();

    for k in 0..cols {
        // check if this column has any non-zero entries
        let mut non_zero = false;
        for p in 0..rows {
            if coefficients[p][k] != 0 {
                non_zero = true;
                break;
            }
        }
        if !non_zero {
            continue; // skip all-zero button
        }

        // build the button from this column
        let mut button = Vec::with_capacity(rows);
        for p in 0..rows {
            button.push(coefficients[p][k]);
        }
        buttons.push(button);
    }

    (buttons, rhs)
}

fn build_system_from_buttons(buttons: Vec<Vec<u32>>, rhs: Vec<u32>) -> (Vec<Vec<u32>>, Vec<u32>) {
    let len = rhs.len();

    let mut coefficients = vec![vec![0_u32; buttons.len()]; len];

    for (k, button) in buttons.iter().enumerate() {
        for (p, &v) in button.iter().enumerate() {
            coefficients[p][k] = v;
        }
    }

    (coefficients, rhs.clone())
}

fn simplify(mut coefficients: Vec<Vec<u32>>, mut rhs: Vec<u32>) -> (Vec<Vec<u32>>, Vec<u32>) {
    let rows = coefficients.len();
    let cols = coefficients[0].len();

    println!("{} | {} |", rows, State { joltages: rhs.clone() });
    for row in &coefficients {
        print!("{} ", State { joltages: row.clone() });
    }
    println!();

    loop {
        let mut changed = false;

        'outer: for i in 0..rows {
            for j in 0..rows {
                if i == j {
                    continue;
                }

                // a[i] must be a superset of a[j]
                let mut superset = true;
                let mut nonzero = false;

                for c in 0..cols {
                    if coefficients[i][c] < coefficients[j][c] {
                        superset = false;
                        break;
                    }
                    if coefficients[j][c] != 0 {
                        nonzero = true;
                    }
                }

                // skip if j is not subset of i OR j is all zeros
                if !superset || !nonzero {
                    continue;
                }

                // rhs[i] must be >= rhs[j]
                if rhs[i] < rhs[j] {
                    continue;
                }

                // do row_i = row_i - row_j
                for c in 0..cols {
                    coefficients[i][c] -= coefficients[j][c];
                }
                rhs[i] -= rhs[j];

                changed = true;
                break 'outer;
            }
        }

        if !changed {
            break;
        }
    }

    (coefficients, rhs)
}

fn extract_trivial_solutions(a: Vec<Vec<u32>>, b: Vec<u32>) -> (Vec<Vec<u32>>, Vec<u32>, u32) {
    let mut remaining_a = Vec::new();
    let mut remaining_b = Vec::new();
    let mut sum = 0_u32;

    for (row, &rhs) in a.iter().zip(b.iter()) {
        let mut count = 0;

        for &v in row.iter() {
            if v != 0 {
                count += 1;
                if count > 1 {
                    break;
                }
            }
        }

        if count == 1 {
            sum += rhs;
            continue;
        }

        remaining_a.push(row.clone());
        remaining_b.push(rhs);
    }

    (remaining_a, remaining_b, sum)
}

struct Solver {
    target: Vec<u32>,
    buttons: Vec<Vec<u32>>,
    len: usize,
}

#[derive(Hash, Eq, PartialEq)]
struct MemoKey {
    current: Vec<u32>,
    remaining_mask: u64,
}

impl Solver {
    fn new(target: Vec<u32>, buttons: Vec<Vec<u32>>) -> Self {
        let len = target.len();
        Self { target, buttons, len }
    }

    fn solve(&self) -> Option<u32> {
        debug_assert!(self.buttons.len() <= 64);
        let ids: Vec<usize> = (0..self.buttons.len()).collect();
        let current = vec![0_u32; self.len];
        let mut memo: HashMap<MemoKey, Option<u32>> = HashMap::new();
        self.go(&ids, &current, &mut memo)
    }

    fn remaining_mask(&self, remaining_buttons: &[usize]) -> u64 {
        let mut mask = 0_u64;
        for &id in remaining_buttons {
            mask |= 1_u64 << id;
        }
        mask
    }

    fn least_frequent(&self, button_ids: &[usize]) -> usize {
        let mut frequency_map = vec![0_u32; self.len];

        for &i in button_ids {
            for (j, v) in self.buttons[i].iter().enumerate() {
                frequency_map[j] += *v;
            }
        }

        frequency_map
            .into_iter()
            .enumerate()
            .filter(|&(_, f)| f > 0)
            .min_by_key(|&(_, f)| f)
            .map(|(i, _)| i)
            .unwrap()
    }

    /// Returns minimal additional presses needed from this state.
    fn go(
        &self,
        remaining_buttons: &[usize],
        current: &[u32],
        memo: &mut HashMap<MemoKey, Option<u32>>,
    ) -> Option<u32> {
        let key = MemoKey {
            current: current.to_vec(),
            remaining_mask: self.remaining_mask(remaining_buttons),
        };

        if let Some(&cached) = memo.get(&key) {
            return cached;
        }

        // base case
        if remaining_buttons.is_empty() {
            let ans = if self.target == current { Some(0) } else { None };
            memo.insert(key, ans);
            return ans;
        }

        let subject_index = self.least_frequent(remaining_buttons);

        let (subject_buttons, remaining_buttons_vec): (Vec<usize>, Vec<usize>) = remaining_buttons
            .iter()
            .copied()
            .partition(|&i| self.buttons[i][subject_index] == 1);

        let remaining_buttons = remaining_buttons_vec;

        let needed = self.target[subject_index].saturating_sub(current[subject_index]);

        // same pruning semantics as your last working version (min over {0,1} includes zeros)
        for i in 0..self.len {
            if i == subject_index {
                continue;
            }

            let space = self.target[i].saturating_sub(current[i]);
            let min_cost = subject_buttons
                .iter()
                .map(|&b_id| self.buttons[b_id][i])
                .min()
                .unwrap_or(0);

            if (needed as u64 * min_cost as u64) > space as u64 {
                memo.insert(key, None);
                return None;
            }
        }

        // per-button max presses
        let maxes = subject_buttons
            .iter()
            .map(|&button_id| {
                self.buttons[button_id]
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| **v > 0)
                    .map(|(i, v)| self.target[i].saturating_sub(current[i]) / *v)
                    .min()
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();

        let mut best: Option<u32> = None;

        for it in Permutations::new(maxes, needed) {
            // build result Vec<u32>
            let mut result = current.to_vec();

            for (idx, &count) in it.iter().enumerate() {
                if count == 0 {
                    continue;
                }
                let b_id = subject_buttons[idx];
                let button = &self.buttons[b_id];

                for d in 0..self.len {
                    result[d] += button[d] * count; // button[d] is 0/1
                }
            }

            // overshoot?
            if (0..self.len).any(|d| self.target[d] < result[d]) {
                continue;
            }

            // must satisfy the chosen dimension exactly
            if result[subject_index] != self.target[subject_index] {
                continue;
            }

            let presses_here = it.iter().sum::<u32>();

            if let Some(extra) = self.go(&remaining_buttons, &result, memo) {
                let total = presses_here + extra;
                best = Some(best.map_or(total, |b| b.min(total)));
            }
        }

        memo.insert(key, best);
        best
    }
}
