#![no_main]

use libfuzzer_sys::fuzz_target;
/// A small demonstration of the Input API.
/// This prints embedded bytes with a custom header and then reads from STDIN.
use bat::{Input, PrettyPrinter};

fuzz_target!(|data: &[u8]| {
    // code to fuzz goes here
    PrettyPrinter::new()
        .header(true)
        .grid(true)
        .line_numbers(true)
        .inputs(vec![
            Input::from_bytes(data)
        ])
        .print()
        .unwrap();
});