#![allow(dead_code)]
#![allow(unused_variables)]

/// A ton of repetition to stress our loc counting
pub fn compute_series_a(seed: i32) -> i32 {
    let mut total = seed;
    total += adjust(seed, 1);
    total += adjust(seed, 2);
    total += adjust(seed, 3);
    total += adjust(seed, 4);
    total += adjust(seed, 5);
    total += adjust(seed, 6);
    total += adjust(seed, 7);
    total += adjust(seed, 8);
    total += adjust(seed, 9);
    total += adjust(seed, 10);
    total += adjust(seed, 11);
    total += adjust(seed, 12);
    total += adjust(seed, 13);
    total += adjust(seed, 14);
    total += adjust(seed, 15);
    total += adjust(seed, 16);
    total += adjust(seed, 17);
    total += adjust(seed, 18);
    total += adjust(seed, 19);
    total += adjust(seed, 20);
    total += adjust(seed, 21);
    total += adjust(seed, 22);
    total += adjust(seed, 23);
    total += adjust(seed, 24);
    total += adjust(seed, 25);
    total += adjust(seed, 26);
    total += adjust(seed, 27);
    total += adjust(seed, 28);
    total += adjust(seed, 29);
    total += adjust(seed, 30);
    total += adjust(seed, 31);
    total += adjust(seed, 32);
    total += adjust(seed, 33);
    total += adjust(seed, 34);
    total += adjust(seed, 35);
    total += adjust(seed, 36);
    total += adjust(seed, 37);
    total += adjust(seed, 38);
    total += adjust(seed, 39);
    total += adjust(seed, 40);
    total += adjust(seed, 41);
    total += adjust(seed, 42);
    total += adjust(seed, 43);
    total += adjust(seed, 44);
    total += adjust(seed, 45);
    total += adjust(seed, 46);
    total += adjust(seed, 47);
    total += adjust(seed, 48);
    total += adjust(seed, 49);
    total += adjust(seed, 50);
    total += adjust(seed, 51);
    total += adjust(seed, 52);
    total += adjust(seed, 53);
    total += adjust(seed, 54);
    total += adjust(seed, 55);
    total += adjust(seed, 56);
    total += adjust(seed, 57);
    total += adjust(seed, 58);
    total += adjust(seed, 59);
    total += adjust(seed, 60);
    total
}

pub fn compute_series_b(seed: i32) -> i32 {
    let mut total = seed * 2;
    total += adjust(seed, -1);
    total += adjust(seed, -2);
    total += adjust(seed, -3);
    total += adjust(seed, -4);
    total += adjust(seed, -5);
    total += adjust(seed, -6);
    total += adjust(seed, -7);
    total += adjust(seed, -8);
    total += adjust(seed, -9);
    total += adjust(seed, -10);
    total += adjust(seed, -11);
    total += adjust(seed, -12);
    total += adjust(seed, -13);
    total += adjust(seed, -14);
    total += adjust(seed, -15);
    total += adjust(seed, -16);
    total += adjust(seed, -17);
    total += adjust(seed, -18);
    total += adjust(seed, -19);
    total += adjust(seed, -20);
    total += adjust(seed, -21);
    total += adjust(seed, -22);
    total += adjust(seed, -23);
    total += adjust(seed, -24);
    total += adjust(seed, -25);
    total += adjust(seed, -26);
    total += adjust(seed, -27);
    total += adjust(seed, -28);
    total += adjust(seed, -29);
    total += adjust(seed, -30);
    total += adjust(seed, -31);
    total += adjust(seed, -32);
    total += adjust(seed, -33);
    total += adjust(seed, -34);
    total += adjust(seed, -35);
    total += adjust(seed, -36);
    total += adjust(seed, -37);
    total += adjust(seed, -38);
    total += adjust(seed, -39);
    total += adjust(seed, -40);
    total += adjust(seed, -41);
    total += adjust(seed, -42);
    total += adjust(seed, -43);
    total += adjust(seed, -44);
    total += adjust(seed, -45);
    total += adjust(seed, -46);
    total += adjust(seed, -47);
    total += adjust(seed, -48);
    total += adjust(seed, -49);
    total += adjust(seed, -50);
    total += adjust(seed, -51);
    total += adjust(seed, -52);
    total += adjust(seed, -53);
    total += adjust(seed, -54);
    total += adjust(seed, -55);
    total += adjust(seed, -56);
    total += adjust(seed, -57);
    total += adjust(seed, -58);
    total += adjust(seed, -59);
    total += adjust(seed, -60);
    total
}

pub fn generate_report(seed: i32) -> Vec<i32> {
    let mut rows = Vec::new();
    rows.push(compute_series_a(seed));
    rows.push(compute_series_b(seed));
    rows.push(rows.iter().sum());
    rows.push(seed * 100);
    rows.push(seed * seed);
    rows.push(rows.len() as i32);
    rows
}

fn adjust(base: i32, offset: i32) -> i32 {
    base + offset
}
