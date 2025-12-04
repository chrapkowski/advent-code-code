use std::{
    fs::File,
    io::{self, BufRead},
    path::Path
};

fn find_max(chars: &[char], offset: usize, length: usize) -> (usize, u32) {
    let mut max = chars.get(offset).unwrap().to_digit(10).unwrap();
    let mut index= offset;

    for i in offset..offset+length {
        let digit = chars.get(i).unwrap().to_digit(10).unwrap();

        if digit > max {
            index = i;
            max = digit;
        }
    }

    (index, max)
}

fn main() -> io::Result<()> {
    let path = Path::new("../input");
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut sum = 0;

    for line in reader.lines() {
        let line = line?;
        let len = line.len();
        let chars: Vec<char> = line.chars().collect();
        let mut joltage = 0_u64;
        let mut offset = 0;
        let n = 2;

        for i in 0..n {
            let remaining = n - i - 1;
            let (start, number) = find_max(&chars, offset, len - offset - remaining);

            joltage = joltage * 10 + number as u64;
            offset = start + 1;
        }

        sum += joltage;
    }

    println!("{sum}");

    Ok(())
}
