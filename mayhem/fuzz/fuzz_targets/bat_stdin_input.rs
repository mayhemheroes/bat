#![no_main]

use libfuzzer_sys::fuzz_target;
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
    highlight_range: Option<(usize, usize)>,
}

fuzz_target!(|input: TestInput<'_>| {
    let mut printer = PrettyPrinter::new();
    printer.inputs(vec![Input::from_bytes(input.data)]);

    // Clamp config-like knobs (terminal geometry, line indices) to realistic
    // ranges: they are environment/config, not attacker input, and unclamped
    // huge values just make bat allocate width-sized padding (harness-inflicted
    // OOM noise that drowns real findings). The DATA stays fully untrusted.
    let input = TestInput {
        term_width: input.term_width.map(|w| (w % 1024).max(1)),
        tab_width: input.tab_width.map(|t| t.map(|t| t % 64)),
        highlight: input.highlight.map(|h| h % 4096),
        highlight_range: input.highlight_range.map(|(a, b)| (a % 4096, b % 4096)),
        ..input
    };

    macro_rules! apply {
        ($x:ident) => {{
            if let Some(val) = input.$x {
                printer.$x(val);
            }
        }};
        ($x:ident, $($y:ident),+) => {{
            apply!($x);
            apply!($($y),+);
        }};
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

    // Render into an in-memory buffer so the harness never writes to a real
    // terminal / spawns a pager while fuzzing.
    let mut out = String::new();
    let _ = printer.print_with_writer(Some(&mut out));
});
