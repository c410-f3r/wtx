#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn unmask(b: &mut Bencher) {
    const DATA_LEN: usize = 64 << 20;
    let mut data: Vec<u8> = (0..DATA_LEN)
        .map(|el| {
            let n = el % usize::try_from(u8::MAX).unwrap();
            n.try_into().unwrap()
        })
        .collect();
    let mask = [3, 5, 7, 11];
    b.iter(|| wtx::web_socket::unmask(&mut data, mask));
}
