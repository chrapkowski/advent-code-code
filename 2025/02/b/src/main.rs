use std::{
    fs::read_to_string,
    io::{self}
};

fn count_invalid(a: u64, b: u64, n: u32) -> u64 {
    if n == 1 {
        return 0;
    }

    let mut sum= 0;

    for i in a..=b {
        for k in 2..=n {
            if n % k != 0 {
                continue
            }

            let m = n/k;
            let t = i / 10_u64.pow(n - m);
            let mut invalid = true;

            for j in 1..k {
                let r = i % 10_u64.pow(n - m * j) / 10_u64.pow(n - m * (j + 1));
                if t != r {
                    invalid = false;
                }
            }

            if invalid {
                sum += i;
                break
            }
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
            let start = if i == m { a } else { 10_u64.pow(i - 1) };
            let end = if i == n { b } else { 10_u64.pow(i) - 1 };
            let invalid = count_invalid(start, end, i);
            sum += invalid;
        }
    }

    println!("{sum}");

    Ok(())
}
