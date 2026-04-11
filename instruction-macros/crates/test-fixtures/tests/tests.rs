// NOTE: `cargo-expand` must be available to run this test.
//
// Ideally `rustfmt` is available, too, otherwise the expanded code won't be formatted.
//
// Note that these tests will fail if there's existing `*.expanded.rs` files that differs from the
// generated output.
// If you expect changes and don't want the tests to fail on any diff, run the test with
// MACROTEST=overwrite. For example:
//
// MACROTEST=overwrite cargo test -p instruction-macros-test-fixtures -- --nocapture 2>&1

#[test]
pub fn expand_client() {
    macrotest::expand_args("src/client.rs", ["--features", "client"]);
}

#[test]
pub fn expand_program() {
    macrotest::expand_args("src/program.rs", ["--features", "program"]);
}

#[test]
pub fn expand_events() {
    macrotest::expand_args("src/events.rs", ["--features", "client"]);
}

#[test]
pub fn expand_pack_and_unpack() {
    macrotest::expand_args("src/pack_and_unpack.rs", ["--features", "no_extra_derives"]);
}

#[test]
pub fn expand_codama() {
    macrotest::expand_args("src/codama.rs", ["--features", "codama"]);
}

#[test]
pub fn expand_codama_events() {
    macrotest::expand_args("src/codama_events.rs", ["--features", "codama"]);
}
