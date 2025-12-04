use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use regex::Regex;

const N: i32 = 100;

fn main() -> io::Result<()> {
    let path = Path::new("../input");
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let re = Regex::new(r"([R|L])(\d+)").unwrap();
    let mut current = 50;
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let direction = &caps[1];
            let steps = &caps[2];
            let direction: i32 = if direction.eq("R") { 1 } else { -1 };
            let steps: i32 = steps.parse().unwrap();

            let first = if current == 0 { N } else { if direction > 0 { N - current } else { current } };

            count += if steps < first { 0 } else { 1 + (steps - first) / N };
            current = (current + direction * steps).rem_euclid(N);
        }
    }

    println!("{count}");

    Ok(())
}
