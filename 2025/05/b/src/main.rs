use std::{
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

fn read_data(path: &Path) -> io::Result<Vec<(u64, u64)>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut intervals = vec![];

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        let range = parse_range(line)?;
        add_range(&mut intervals, &range);
    }

    Ok(intervals)
}

fn main() -> io::Result<()> {
    let intervals = read_data(Path::new("../input"))?;
    let sum: u64 = intervals.iter().map(|it| it.1 - it.0 + 1).sum();

    println!("{sum}");

    Ok(())
}
