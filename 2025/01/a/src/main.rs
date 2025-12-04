use std::{
    fs::File,
    io::{self, BufRead},
    path::Path
};

use regex::Regex;

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
            let value = &caps[2];
            let direction: i32 = if direction.eq("R") { 1 } else { -1 };
            let value: i32 = value.parse().unwrap();
            let value = direction * value;
            let previous = current;

            current = (previous + value) % 100;

            if current == 0 {
                count += 1;
            }
        }
    }

    println!("{count}");

    Ok(())
}
