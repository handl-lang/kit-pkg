//! # uuid_c.rs — `uuid-c` and `uuid-stub` end-to-end smoke tests.
//!
//! `uuid-c` loads directly from `handl-packages/uuid-c` and draws 16 random
//! bytes from `rand-c`. `uuid-stub` is the deterministic, no-effect backend.

use std::path::PathBuf;

use handl_kit_prelude::Kit;
use handl_kit_pkg::KitProjectExt;

/// The workspace root — `new_with_project` wires its `.handl.toml`
/// `[dependencies]` into the module loader, so a package's own name imports
/// (e.g. `from "rand-c"` inside `uuid-c.hnl`) resolve to the sibling repos.
fn kit() -> Kit {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    Kit::new_with_project(root).expect("kit must build")
}

fn eval_uuid_c(input: &str) -> serde_json::Value {
    let kit = kit();
    let lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lib-uuid-c/src");
    let src = format!(
        "do {{\n  from \"{}\" import {{ v4 }};\n  {}\n}}",
        lib.display(),
        input
    );
    kit.eval(&src).expect("eval must succeed")
}

fn eval_uuid_stub(input: &str) -> serde_json::Value {
    let kit = kit();
    let lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lib-uuid-stub/src");
    let src = format!(
        "do {{\n  from \"{}\" import {{ v4 }};\n  {}\n}}",
        lib.display(),
        input
    );
    kit.eval(&src).expect("eval must succeed")
}

// ---------------------------------------------------------------------------
// uuid-c — v4 UUID generation (loaded from file)
// ---------------------------------------------------------------------------

#[test]
fn uuid_c_v4_format_is_valid() {
    let v = eval_uuid_c("v4()");
    let s = v.as_str().expect("v4 must return a string");
    assert_eq!(s.len(), 36, "uuid-c v4 must be 36 characters: {s}");
    assert_eq!(s.chars().nth(8), Some('-'), "hyphen at 8");
    assert_eq!(s.chars().nth(13), Some('-'), "hyphen at 13");
    assert_eq!(s.chars().nth(18), Some('-'), "hyphen at 18");
    assert_eq!(s.chars().nth(23), Some('-'), "hyphen at 23");
    assert_eq!(
        s.chars().nth(14),
        Some('4'),
        "uuid-c v4 version must be 4: {s}"
    );
    let variant = s.chars().nth(19).expect("fourth segment must have a char");
    assert!(
        matches!(variant, '8' | '9' | 'a' | 'b'),
        "uuid-c v4 variant must be 8/9/a/b: {s}"
    );
}

// ---------------------------------------------------------------------------
// uuid-stub — deterministic, no-effect backend
// ---------------------------------------------------------------------------

#[test]
fn uuid_stub_is_deterministic() {
    let a = eval_uuid_stub("v4()");
    let b = eval_uuid_stub("v4()");
    assert_eq!(a, b, "uuid-stub v4 must be deterministic");
    assert_eq!(a, serde_json::json!("00000000-0000-4000-8000-000000000000"));
}
