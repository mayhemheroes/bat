#![no_main]

/// A small demonstration of the Input API.
/// This prints embedded bytes with a custom header and then reads from STDIN.
use bat::PrettyPrinter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|printer: PrettyPrinter<'_>| {
    let mut printer = printer;
    // code to fuzz goes here
    printer.print().unwrap();
});
