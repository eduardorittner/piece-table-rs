extern crate criterion;
extern crate rand;
extern crate ropey;

use criterion::{Criterion, criterion_group, criterion_main};
use piece_table::PieceTable;
use rand::random;
use ropey::Rope;

const TEXT: &str = include_str!("large.txt");
const TEXT_SMALL: &str = include_str!("small.txt");

fn mul_string_length(text: &str, n: usize) -> String {
    let mut mtext = String::new();
    for _ in 0..n {
        mtext.push_str(text);
    }
    mtext
}

//----

const LEN_MUL_SMALL: usize = 1;

fn rope_remove_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_remove_small");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 1).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = 0;
            let end = (start + 1).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = len / 2;
            let end = (start + 1).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let end = len;
            let start = end - (1).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });
}

fn string_remove_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_remove_small");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 1).min(len);
            string.replace_range(start..end, "");

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = 0;
            let end = (start + 1).min(len);
            string.replace_range(start..end, "");

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = len / 2;
            let end = (start + 1).min(len);
            string.replace_range(start..end, "");

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let end = len;
            let start = end - (1).min(len);
            string.replace_range(start..end, "");

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });
}

fn ptable_remove_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_remove_small");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 1).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = 0;
            let end = (start + 1).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = len / 2;
            let end = (start + 1).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_SMALL);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let end = len;
            let start = end - (1).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });
}

const LEN_MUL_MEDIUM: usize = 1;

fn rope_remove_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_remove_medium");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 15).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = 0;
            let end = (start + 15).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = len / 2;
            let end = (start + 15).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let end = len;
            let start = end - (15).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == TEXT.len() / 2 {
                rope = Rope::from_str(&text);
            }
        })
    });
}

fn string_remove_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_remove_medium");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 15).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = 0;
            let end = (start + 15).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = len / 2;
            let end = (start + 15).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let end = len;
            let start = end - (15).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == TEXT.len() / 2 {
                string = text.clone();
            }
        })
    });
}

fn ptable_remove_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_remove_medium");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 15).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = 0;
            let end = (start + 15).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = len / 2;
            let end = (start + 15).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_MEDIUM);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let end = len;
            let start = end - (15).min(len);
            ptable.delete(start..end);

            if ptable.len() == TEXT.len() / 2 {
                ptable = PieceTable::new(&text);
            }
        })
    });
}

const LEN_MUL_LARGE: usize = 4;

fn rope_remove_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_remove_large");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + TEXT_SMALL.len()).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == 0 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = 0;
            let end = (start + TEXT_SMALL.len()).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == 0 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let start = len / 2;
            let end = (start + TEXT_SMALL.len()).min(len);
            rope.remove(start..end);

            if rope.len_bytes() == 0 {
                rope = Rope::from_str(&text);
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut rope = Rope::from_str(&text);

        bench.iter(|| {
            let len = rope.len_chars();
            let end = len;
            let start = end - TEXT_SMALL.len().min(len);
            rope.remove(start..end);

            if rope.len_bytes() == 0 {
                rope = Rope::from_str(&text);
            }
        })
    });
}

fn string_remove_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_remove_large");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + TEXT_SMALL.len()).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == 0 {
                string = text.clone();
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = 0;
            let end = (start + TEXT_SMALL.len()).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == 0 {
                string = text.clone();
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let start = len / 2;
            let end = (start + TEXT_SMALL.len()).min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == 0 {
                string = text.clone();
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut string = text.clone();

        bench.iter(|| {
            let len = string.len();
            let end = len;
            let start = end - TEXT_SMALL.len().min(len);
            for _ in 0..(end - start) {
                if start < string.len() && end <= string.len() {
                    string.replace_range(start..end, "");
                }
            }

            if string.len() == 0 {
                string = text.clone();
            }
        })
    });
}

fn ptable_remove_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_remove_large");

    group.bench_function("random", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + TEXT_SMALL.len()).min(len);
            ptable.delete(start..end);

            if ptable.len() == 0 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("start", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = 0;
            let end = (start + TEXT_SMALL.len()).min(len);
            ptable.delete(start..end);

            if ptable.len() == 0 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("middle", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let start = len / 2;
            let end = (start + TEXT_SMALL.len()).min(len);
            ptable.delete(start..end);

            if ptable.len() == 0 {
                ptable = PieceTable::new(&text);
            }
        })
    });

    group.bench_function("end", |bench| {
        let text = mul_string_length(TEXT, LEN_MUL_LARGE);
        let mut ptable = PieceTable::new(&text);

        bench.iter(|| {
            let len = ptable.len();
            let end = len;
            let start = end - TEXT_SMALL.len().min(len);
            ptable.delete(start..end);

            if ptable.len() == 0 {
                ptable = PieceTable::new(&text);
            }
        })
    });
}

fn rope_remove_initial_after_clone(c: &mut Criterion) {
    c.bench_function("rope_remove_initial_after_clone", |bench| {
        let rope = Rope::from_str(TEXT);
        let mut rope_clone = rope.clone();
        let mut i = 0;
        bench.iter(|| {
            if i > 32 {
                i = 0;
                rope_clone = rope.clone();
            }
            let len = rope_clone.len_chars();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 1).min(len);
            rope_clone.remove(start..end);
            i += 1;
        })
    });
}

fn string_remove_initial_after_clone(c: &mut Criterion) {
    c.bench_function("string_remove_initial_after_clone", |bench| {
        let string = String::from(TEXT);
        let mut string_clone = string.clone();
        let mut i = 0;
        bench.iter(|| {
            if i > 32 {
                i = 0;
                string_clone = string.clone();
            }
            let len = string_clone.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 1).min(len);
            if start < string_clone.len() {
                string_clone.replace_range(start..end, "");
            }
            i += 1;
        })
    });
}

fn ptable_remove_initial_after_clone(c: &mut Criterion) {
    c.bench_function("ptable_remove_initial_after_clone", |bench| {
        let ptable = PieceTable::new(TEXT);
        let mut ptable_clone = ptable.clone();
        let mut i = 0;
        bench.iter(|| {
            if i > 32 {
                i = 0;
                ptable_clone = ptable.clone();
            }
            let len = ptable_clone.len();
            let start = random::<u64>() as usize % (len + 1);
            let end = (start + 1).min(len);
            ptable_clone.delete(start..end);
            i += 1;
        })
    });
}

//----

criterion_group!(
    benches,
    rope_remove_small,
    string_remove_small,
    ptable_remove_small,
    rope_remove_medium,
    string_remove_medium,
    ptable_remove_medium,
    rope_remove_large,
    string_remove_large,
    ptable_remove_large,
    rope_remove_initial_after_clone,
    string_remove_initial_after_clone,
    ptable_remove_initial_after_clone
);
criterion_main!(benches);
