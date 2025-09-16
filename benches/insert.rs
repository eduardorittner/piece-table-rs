extern crate criterion;
extern crate rand;
extern crate ropey;

use criterion::{Criterion, criterion_group, criterion_main};
use piece_table::PieceTable;
use rand::random;
use ropey::Rope;

const TEXT: &str = include_str!("large.txt");

//----

fn rope_insert_char(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_insert_char");

    group.bench_function("random", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert_char(random::<u64>() as usize % len, 'a')
        })
    });

    group.bench_function("start", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.insert_char(0, 'a');
        })
    });

    group.bench_function("middle", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert_char(len / 2, 'a');
        })
    });

    group.bench_function("end", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert_char(len, 'a');
        })
    });
}

fn string_insert_char(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_insert_char");

    group.bench_function("random", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert(random::<u64>() as usize % len, 'a')
        })
    });

    group.bench_function("start", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            string.insert(0, 'a');
        })
    });

    group.bench_function("middle", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert(len / 2, 'a');
        })
    });

    group.bench_function("end", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert(len, 'a');
        })
    });
}

fn ptable_insert_char(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_insert_char");

    group.bench_function("random", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert_char(random::<u64>() as usize % len, 'a')
        })
    });

    group.bench_function("start", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            ptable.insert_char(0, 'a');
        })
    });

    group.bench_function("middle", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert_char(len / 2, 'a');
        })
    });

    group.bench_function("end", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert_char(len, 'a');
        })
    });
}

fn rope_insert_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_insert_small");

    group.bench_function("random", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(random::<u64>() as usize % len, "a");
        })
    });

    group.bench_function("start", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.insert(0, "a");
        })
    });

    group.bench_function("middle", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(len / 2, "a");
        })
    });

    group.bench_function("end", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(len, "a");
        })
    });
}

fn string_insert_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_insert_small");

    group.bench_function("random", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(random::<u64>() as usize % len, "a");
        })
    });

    group.bench_function("start", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            string.insert_str(0, "a");
        })
    });

    group.bench_function("middle", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(len / 2, "a");
        })
    });

    group.bench_function("end", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(len, "a");
        })
    });
}

fn ptable_insert_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_insert_small");

    group.bench_function("random", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert("a", random::<u64>() as usize % len);
        })
    });

    group.bench_function("start", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            ptable.insert("a", 0);
        })
    });

    group.bench_function("middle", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert("a", len / 2);
        })
    });

    group.bench_function("end", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert("a", len);
        })
    });
}

fn rope_insert_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_insert_medium");

    group.bench_function("random", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(random::<u64>() as usize % len, "This is some text.");
        })
    });

    group.bench_function("start", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.insert(0, "This is some text.");
        })
    });

    group.bench_function("middle", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(len / 2, "This is some text.");
        })
    });

    group.bench_function("end", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(len, "This is some text.");
        })
    });
}

fn string_insert_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_insert_medium");

    group.bench_function("random", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(random::<u64>() as usize % len, "This is some text.");
        })
    });

    group.bench_function("start", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            string.insert_str(0, "This is some text.");
        })
    });

    group.bench_function("middle", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(len / 2, "This is some text.");
        })
    });

    group.bench_function("end", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(len, "This is some text.");
        })
    });
}

fn ptable_insert_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_insert_medium");

    group.bench_function("random", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert("This is some text.", random::<u64>() as usize % len);
        })
    });

    group.bench_function("start", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            ptable.insert("This is some text.", 0);
        })
    });

    group.bench_function("middle", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert("This is some text.", len / 2);
        })
    });

    group.bench_function("end", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert("This is some text.", len);
        })
    });
}

const INSERT_TEXT: &str = include_str!("small.txt");

fn rope_insert_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_insert_large");

    group.bench_function("random", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(random::<u64>() as usize % len, INSERT_TEXT);
        })
    });

    group.bench_function("start", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.insert(0, INSERT_TEXT);
        })
    });

    group.bench_function("middle", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(len / 2, INSERT_TEXT);
        })
    });

    group.bench_function("end", |bench| {
        let mut rope = Rope::from_str(TEXT);
        bench.iter(|| {
            let len = rope.len_chars();
            rope.insert(len, INSERT_TEXT);
        })
    });
}

fn string_insert_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_insert_large");

    group.bench_function("random", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(random::<u64>() as usize % len, INSERT_TEXT);
        })
    });

    group.bench_function("start", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            string.insert_str(0, INSERT_TEXT);
        })
    });

    group.bench_function("middle", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(len / 2, INSERT_TEXT);
        })
    });

    group.bench_function("end", |bench| {
        let mut string = String::from(TEXT);
        bench.iter(|| {
            let len = string.len();
            string.insert_str(len, INSERT_TEXT);
        })
    });
}

fn ptable_insert_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_insert_large");

    group.bench_function("random", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert(INSERT_TEXT, random::<u64>() as usize % len);
        })
    });

    group.bench_function("start", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            ptable.insert(INSERT_TEXT, 0);
        })
    });

    group.bench_function("middle", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert(INSERT_TEXT, len / 2);
        })
    });

    group.bench_function("end", |bench| {
        let mut ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            let len = ptable.len();
            ptable.insert(INSERT_TEXT, len);
        })
    });
}

//----

fn insert_after_clone(c: &mut Criterion) {
    c.bench_function("insert_after_clone", |bench| {
        let rope = Rope::from_str(TEXT);
        let mut rope_clone = rope.clone();
        let mut i = 0;
        bench.iter(|| {
            if i > 32 {
                i = 0;
                rope_clone = rope.clone();
            }
            let len = rope_clone.len_chars();
            rope_clone.insert(random::<u64>() as usize % len, "a");
            i += 1;
        })
    });
}

//----

criterion_group!(
    benches,
    rope_insert_char,
    string_insert_char,
    ptable_insert_char,
    rope_insert_small,
    string_insert_small,
    ptable_insert_small,
    rope_insert_medium,
    string_insert_medium,
    ptable_insert_medium,
    rope_insert_large,
    string_insert_large,
    ptable_insert_large,
    insert_after_clone
);
criterion_main!(benches);
