use std::{
    cmp::Ordering,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

fn add_range(intervals: &mut Vec<(u64, u64)>, range: &(u64, u64)) {
    let i = intervals.partition_point(|it| it.1 < range.0);
    let j = intervals.partition_point(|it| it.0 <= range.1);

    if i == j {
        intervals.insert(i, *range);
        return;
    }

    let new_start = range.0.min(intervals[i].0);
    let new_end = range.1.max(intervals[j - 1].1);
    intervals.splice(i..j, [(new_start, new_end)]);
}

fn contains(intervals: &[(u64, u64)], value: u64) -> bool {
    intervals
        .binary_search_by(|it| {
            if it.1 < value {
                Ordering::Less
            } else if it.0 > value {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
        .is_ok()
}

fn parse_range(line: &str) -> io::Result<(u64, u64)> {
    let mut parts = line.splitn(2, '-');
    let (Some(a), Some(b)) = (parts.next(), parts.next()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid range format `{line}`"),
        ));
    };

    let start = a.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid range start `{a}` in `{line}`"),
        )
    })?;
    let end = b.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid range end `{b}` in `{line}`"),
        )
    })?;

    Ok((start, end))
}

fn read_data(path: &Path) -> io::Result<(Vec<(u64, u64)>, Vec<u64>)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut intervals = vec![];
    let mut numbers = vec![];
    let mut interval_mode = true;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            interval_mode = false;
            continue;
        }

        if interval_mode {
            let range = parse_range(line)?;
            add_range(&mut intervals, &range);
            continue;
        }

        let number = line
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("invalid number `{line}`")))?;
        numbers.push(number);
    }

    Ok((intervals, numbers))
}

fn main() -> io::Result<()> {
    let (intervals, numbers) = read_data(Path::new("../input"))?;
    let count = numbers.iter().filter(|number| contains(&intervals, **number)).count();

    println!("{count}");

    Ok(())
}
