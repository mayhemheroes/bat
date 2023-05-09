#![no_main]

use libfuzzer_sys::fuzz_target;
/// A small demonstration of the Input API.
/// This prints embedded bytes with a custom header and then reads from STDIN.
use bat::{Input, PrettyPrinter};

use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct TestInput<'a> {
    data: &'a [u8],
    term_width: Option<usize>,
    tab_width: Option<Option<usize>>,
    colored_output: Option<bool>,
    true_color: Option<bool>,
    header: Option<bool>,
    line_numbers: Option<bool>,
    grid: Option<bool>,
    rule: Option<bool>,
    vcs_modification_markers: Option<bool>,
    show_nonprintable: Option<bool>,
    snip: Option<bool>,
    use_italics: Option<bool>,
    pager: Option<&'a str>,
    highlight: Option<usize>,
    highlight_range: Option<(usize, usize)>
}

fuzz_target!(|input: TestInput<'_>| {
    // code to fuzz goes here
    let mut printer = PrettyPrinter::new();
    printer.inputs(vec![
        Input::from_bytes(input.data)
    ]);

    macro_rules! apply {
        ($x:ident) => {{
            if let Some(val) = input.$x {
                printer.$x(val);
            }
        }};
        ($x:ident, $($y:ident),+) => {{
            apply!($x);
            apply!($($y),+);
        }}
    }

    apply!(
        term_width,
        tab_width,
        colored_output,
        true_color,
        header,
        line_numbers,
        grid,
        rule,
        vcs_modification_markers,
        show_nonprintable,
        snip,
        use_italics,
        pager,
        highlight
    );

    if let Some((from, to)) = input.highlight_range {
        printer.highlight_range(from, to);
    }

    printer
        .print()
        .unwrap();
});
