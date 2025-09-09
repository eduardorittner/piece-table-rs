use criterion::{black_box, criterion_group, criterion_main, Criterion};
use piece_table::baseline::Baseline;
use piece_table::interface::EditableText;
use piece_table::{PieceTable, TextRange};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
enum Op {
    Insert(String, usize),
    Delete(usize, usize),
}

fn load_workload(path: &str) -> Vec<Op> {
    let file = File::open(path).expect("Failed to open workload file");
    let reader = BufReader::new(file);
    let mut ops = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        match parts[0] {
            "INSERT" => {
                let offset = parts[1].parse().expect("Failed to parse offset");
                ops.push(Op::Insert(parts[2].to_string(), offset));
            }
            "DELETE" => {
                let start = parts[1].parse().expect("Failed to parse start offset");
                let end = parts[2].parse().expect("Failed to parse end offset");
                ops.push(Op::Delete(start, end));
            }
            _ => panic!("Unknown operation"),
        }
    }

    ops
}

fn editing_benchmark(c: &mut Criterion) {
    let workload = load_workload("workloads/mostly_inserts.txt");

    c.bench_function("piece_table_mostly_inserts", |b| {
        b.iter(|| {
            let mut table = PieceTable::new(String::new());
            for op in &workload {
                match op {
                    Op::Insert(text, offset) => table.insert(black_box(text), black_box(*offset)),
                    Op::Delete(start, end) => table.delete(black_box(TextRange { start: *start, end: *end })),
                }
            }
        })
    });

    c.bench_function("baseline_mostly_inserts", |b| {
        b.iter(|| {
            let mut baseline = Baseline::new(String::new());
            for op in &workload {
                match op {
                    Op::Insert(text, offset) => baseline.insert(black_box(text), black_box(*offset)),
                    Op::Delete(start, end) => baseline.delete(black_box(TextRange { start: *start, end: *end })),
                }
            }
        })
    });
}

criterion_group!(benches, editing_benchmark);
criterion_main!(benches);
