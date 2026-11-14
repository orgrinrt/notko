//! Ad-hoc spike, not a bench: one map over rows, `c = a * 3 + b`, written
//! against a product of columns from `notko-hlist` and against plain slices.
//!
//! The product arms use only what the crate offers for a row: `read_row`,
//! `write_row` and `SubOf::put_row`. The slice arms are what a hand would
//! write for the same three columns.

use std::hint::black_box;
use std::time::Instant;

use notko_hlist::{Arr, ColMut, Cons, Elem, Empty, PCons, PNil, Project, SubOf};

pub struct A;
impl Elem for A {
    type Value = u32;
}
pub struct B;
impl Elem for B {
    type Value = u32;
}
pub struct C;
impl Elem for C {
    type Value = u32;
}

type L = Cons<A, Cons<B, Cons<C, Empty>>>;
type W = Cons<C, Empty>;
type Cols<'a> = <L as Project>::Prod<ColMut<'a>>;

#[inline(always)]
fn kernel(a: u32, b: u32) -> u32 {
    a.wrapping_mul(3).wrapping_add(b)
}

/// The idiom: three slices zipped.
#[inline(never)]
fn zip(a: &[u32], b: &[u32], c: &mut [u32]) {
    for ((a, b), c) in a.iter().zip(b).zip(c.iter_mut()) {
        *c = kernel(*a, *b);
    }
}

/// Indexed, with the lengths asserted equal once before the loop.
#[inline(never)]
fn indexed(a: &[u32], b: &[u32], c: &mut [u32]) {
    let n = c.len();
    assert!(a.len() == n && b.len() == n);
    for i in 0 .. n {
        c[i] = kernel(a[i], b[i]);
    }
}

/// No checks at all.
#[inline(never)]
fn unchecked(a: &[u32], b: &[u32], c: &mut [u32]) {
    let n = c.len();
    assert!(a.len() == n && b.len() == n);
    for i in 0 .. n {
        // SAFETY: the three lengths were asserted equal to `n` above.
        unsafe {
            *c.get_unchecked_mut(i) = kernel(*a.get_unchecked(i), *b.get_unchecked(i));
        }
    }
}

/// The product, whole rows: read all three members, write all three back.
#[inline(never)]
fn product_whole_rows(cols: &mut Cols<'_>, n: usize) {
    for i in 0 .. n {
        let r = L::read_row::<ColMut<'_>>(cols, i);
        let out = PCons {
            head: r.head,
            tail: PCons {
                head: r.tail.head,
                tail: PCons {
                    head: kernel(r.head, r.tail.head),
                    tail: PNil,
                },
            },
        };
        L::write_row::<ColMut<'_>>(cols, i, &out);
    }
}

/// The product, a whole row read and the one written member put back through
/// `SubOf`.
#[inline(never)]
fn product_read_row_put_one(cols: &mut Cols<'_>, n: usize) {
    for i in 0 .. n {
        let r = L::read_row::<ColMut<'_>>(cols, i);
        let out = PCons {
            head: kernel(r.head, r.tail.head),
            tail: PNil,
        };
        <W as SubOf<L, _>>::put_row::<ColMut<'_>>(cols, i, &out);
    }
}

/// The product taken apart by hand into its three slices, then zipped. The
/// fields are public, so this is open to a consumer, and it is positional.
#[inline(never)]
fn product_destructured(cols: &mut Cols<'_>) {
    let PCons {
        head: a,
        tail: PCons {
            head: b,
            tail: PCons {
                head: c,
                tail: PNil,
            },
        },
    } = cols;
    for ((a, b), c) in a.iter().zip(b.iter()).zip(c.iter_mut()) {
        *c = kernel(*a, *b);
    }
}

/// The product over owned arrays, whole rows, where the row count is a
/// constant the compiler can see.
#[inline(never)]
fn product_arrays<const N: usize>(cols: &mut <L as Project>::Prod<Arr<N>>) {
    for i in 0 .. N {
        let r = L::read_row::<Arr<N>>(cols, i);
        let out = PCons {
            head: r.head,
            tail: PCons {
                head: r.tail.head,
                tail: PCons {
                    head: kernel(r.head, r.tail.head),
                    tail: PNil,
                },
            },
        };
        L::write_row::<Arr<N>>(cols, i, &out);
    }
}

const SAMPLES: usize = 11;

fn report(name: &str, n: usize, mut pass: impl FnMut()) {
    let passes = ((1usize << 24) / n).max(1);
    pass();
    let mut samples = [0f64; SAMPLES];
    for s in &mut samples {
        let t = Instant::now();
        for _ in 0 .. passes {
            pass();
        }
        *s = t.elapsed().as_nanos() as f64 / (passes * n) as f64;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("| {name} | {n} | {:.3} | {:.3} | {:.3} |", samples[0], samples[SAMPLES / 2], samples[SAMPLES - 1]);
}

fn fill(v: &mut [u32], mut s: u32) {
    for d in v {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        *d = s;
    }
}

fn main() {
    println!("map over rows, c = a * 3 + b, three u32 columns; nanoseconds per row, {SAMPLES} samples");
    println!();
    println!("| arm | rows | min | median | max |");
    println!("|---|---|---|---|---|");

    for &n in &[1usize << 10, 1 << 16, 1 << 22] {
        let mut a = vec![0u32; n];
        let mut b = vec![0u32; n];
        let mut c = vec![0u32; n];
        fill(&mut a, 0x9e37_79b9);
        fill(&mut b, 0x85eb_ca6b);

        report("slices, zip", n, || zip(black_box(&a), black_box(&b), black_box(&mut c)));
        report("slices, indexed, lengths asserted", n, || indexed(black_box(&a), black_box(&b), black_box(&mut c)));
        report("slices, unchecked", n, || unchecked(black_box(&a), black_box(&b), black_box(&mut c)));
        let check = c.iter().fold(0u32, |s, x| s.wrapping_add(*x));

        report("product, read_row and write_row", n, || {
            let mut cols: Cols<'_> = PCons {
                head: &mut a[..],
                tail: PCons {
                    head: &mut b[..],
                    tail: PCons {
                        head: &mut c[..],
                        tail: PNil,
                    },
                },
            };
            product_whole_rows(black_box(&mut cols), black_box(n));
        });
        report("product, read_row and put_row of one member", n, || {
            let mut cols: Cols<'_> = PCons {
                head: &mut a[..],
                tail: PCons {
                    head: &mut b[..],
                    tail: PCons {
                        head: &mut c[..],
                        tail: PNil,
                    },
                },
            };
            product_read_row_put_one(black_box(&mut cols), black_box(n));
        });
        report("product, destructured by hand, zip", n, || {
            let mut cols: Cols<'_> = PCons {
                head: &mut a[..],
                tail: PCons {
                    head: &mut b[..],
                    tail: PCons {
                        head: &mut c[..],
                        tail: PNil,
                    },
                },
            };
            product_destructured(black_box(&mut cols));
        });
        let check2 = c.iter().fold(0u32, |s, x| s.wrapping_add(*x));
        assert_eq!(check, check2, "the arms disagree");
    }

    // Owned arrays, where the row count is a constant.
    {
        const N: usize = 1 << 10;
        let row = PCons {
            head: 7u32,
            tail: PCons {
                head: 11u32,
                tail: PCons {
                    head: 0u32,
                    tail: PNil,
                },
            },
        };
        let mut cols = L::made::<N>(&row);
        fill(&mut cols.head, 0x9e37_79b9);
        fill(&mut cols.tail.head, 0x85eb_ca6b);
        report("product over Arr<N>, read_row and write_row", N, || product_arrays::<N>(black_box(&mut cols)));
    }
}
