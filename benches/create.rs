extern crate criterion;
extern crate ropey;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use piece_table::PieceTable;
use ropey::Rope;

const TEXT_SMALL: &str = include_str!("small.txt");
const TEXT_MEDIUM: &str = include_str!("medium.txt");
const TEXT_LARGE: &str = include_str!("large.txt");
const TEXT_LF: &str = include_str!("lf.txt");

//----

fn rope_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_from_str");

    group.bench_function("small", |bench| {
        bench.iter(|| {
            Rope::from_str(black_box(TEXT_SMALL));
        })
    });

    group.bench_function("medium", |bench| {
        bench.iter(|| {
            Rope::from_str(black_box(TEXT_MEDIUM));
        })
    });

    group.bench_function("large", |bench| {
        bench.iter(|| {
            Rope::from_str(black_box(TEXT_LARGE));
        })
    });

    group.bench_function("linefeeds", |bench| {
        bench.iter(|| {
            Rope::from_str(black_box(TEXT_LF));
        })
    });
}

fn string_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_from_str");

    group.bench_function("small", |bench| {
        bench.iter(|| {
            let _ = String::from(black_box(TEXT_SMALL));
        })
    });

    group.bench_function("medium", |bench| {
        bench.iter(|| {
            let _ = String::from(black_box(TEXT_MEDIUM));
        })
    });

    group.bench_function("large", |bench| {
        bench.iter(|| {
            let _ = String::from(black_box(TEXT_LARGE));
        })
    });

    group.bench_function("linefeeds", |bench| {
        bench.iter(|| {
            let _ = String::from(black_box(TEXT_LF));
        })
    });
}

fn ptable_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_from_str");

    group.bench_function("small", |bench| {
        bench.iter(|| {
            PieceTable::new(black_box(TEXT_SMALL));
        })
    });

    group.bench_function("medium", |bench| {
        bench.iter(|| {
            PieceTable::new(black_box(TEXT_MEDIUM));
        })
    });

    group.bench_function("large", |bench| {
        bench.iter(|| {
            PieceTable::new(black_box(TEXT_LARGE));
        })
    });

    group.bench_function("linefeeds", |bench| {
        bench.iter(|| {
            PieceTable::new(black_box(TEXT_LF));
        })
    });
}

fn rope_clone(c: &mut Criterion) {
    let rope = Rope::from_str(TEXT_LARGE);
    c.bench_function("rope_clone", |bench| {
        bench.iter(|| {
            let _ = black_box(&rope).clone();
        })
    });
}

fn ptable_clone(c: &mut Criterion) {
    let ptable = PieceTable::new(TEXT_LARGE);
    c.bench_function("ptable_clone", |bench| {
        bench.iter(|| {
            let _ = black_box(&ptable).clone();
        })
    });
}

//----

criterion_group!(
    benches,
    rope_from_str,
    string_from_str,
    ptable_from_str,
    rope_clone,
    ptable_clone
);
criterion_main!(benches);
