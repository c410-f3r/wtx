//! Unmask

#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut data = data.to_vec();
    wtx::web_socket::unmask(&mut data, [1, 2, 3, 4]);
});
