#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn bench_clone(b: &mut Bencher) {
    let data = "data".to_string();
    b.iter(move || {
        let _data = data.clone();
    });
}

#[bench]
fn bench_ref(b: &mut Bencher) {
    let data = "data".to_string();
    b.iter(move || {
        let _data = &data;
    });
}
