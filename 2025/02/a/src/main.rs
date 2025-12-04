use std::{
    fs::read_to_string,
    io::{self}
};

fn count_invalid(a: u64, b: u64, n: u32) -> u64 {
    let m = n / 2;
    let mut sum= 0;

    for i in a..=b {
        let d = i / 10_u64.pow(m);
        let e = i % 10_u64.pow(m);

        if d == e {
            sum += i;
        }
    }

    sum
}

fn main() -> io::Result<()> {
    let contents = read_to_string("../input")?;
    let mut sum = 0;
    
    for part in contents.trim().split(',') {
        let mut iter= part.split("-");
        let a = iter.next().unwrap();
        let b = iter.next().unwrap();
        let m = a.len() as u32;
        let n = b.len() as u32;
        let a: u64 = a.parse().unwrap();
        let b: u64 = b.parse().unwrap();

        for i in m..=n {
            if i % 2 != 0 {
                continue
            }

            let start = if i == m { a } else { 10_u64.pow(i - 1) };
            let end = if i == n { b } else { 10_u64.pow(i) - 1 };
            let invalid = count_invalid(start, end, i);
            sum += invalid;
        }
    }

    println!("{sum}");

    Ok(())
}
