use std::fs;

use ssi_purity::filter_declared_deps;
use ssi_purity::scan_vendored_tree;
use ssi_purity::{parse_deps_toml, DeclaredDep, TRUSTED_DIRECT_DEPENDENCIES};

#[test]
fn missing_deps_toml_yields_empty() {
    let dir = std::env::temp_dir().join("ssi-purity-test-nodeps");
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join("deps.toml"));
    assert!(filter_declared_deps(&dir).unwrap().is_empty());
}

#[test]
fn present_deps_toml_is_parsed() {
    let dir = std::env::temp_dir().join("ssi-purity-test-withdeps");
    let _ = fs::create_dir_all(&dir);
    fs::write(
        dir.join("deps.toml"),
        "[dependencies]\nferal-amd = \"0.2.1\"\n",
    )
    .unwrap();
    let got = filter_declared_deps(&dir).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "feral-amd");
}

#[test]
fn allows_every_reviewed_exact_entry() {
    let src = "[dependencies]\n\
               feral = \"0.11.0\"\n\
               feral-amd = \"0.2.1\"\n\
               feral-amf = \"0.2.1\"\n\
               feral-kahip = \"0.2.1\"\n\
               feral-metis = \"0.2.1\"\n\
               feral-ordering-core = \"0.2.1\"\n\
               feral-scotch = \"0.2.1\"\n";
    let got = parse_deps_toml(src).expect("valid");
    let expected = TRUSTED_DIRECT_DEPENDENCIES
        .iter()
        .map(|(name, version)| DeclaredDep {
            name: (*name).into(),
            version: (*version).into(),
        })
        .collect::<Vec<_>>();
    assert_eq!(got, expected);
}

#[test]
fn disallowed_crate_name_has_actionable_error() {
    let err = parse_deps_toml("[dependencies]\nrand = \"0.8.5\"\n").unwrap_err();
    assert!(err
        .0
        .contains("`rand` is not in the reviewed direct-dependency allowlist"));
    assert!(err.0.contains("ask a maintainer to review and approve"));
}

#[test]
fn wrong_reviewed_crate_version_has_actionable_error() {
    let err = parse_deps_toml("[dependencies]\nferal-amd = \"0.2.2\"\n").unwrap_err();
    assert!(err
        .0
        .contains("version `0.2.2` for `feral-amd` is not reviewed"));
    assert!(err.0.contains("feral-amd = \"0.2.1\""));
    assert!(err.0.contains("ask a maintainer to review the upgrade"));
}

#[test]
fn crate_names_may_use_hyphens_and_underscores() {
    let got = parse_deps_toml("[dependencies]\nferal-amd = \"0.2.1\"\n")
        .expect("a reviewed hyphenated crate name should be valid");
    assert_eq!(got[0].name, "feral-amd");

    let err = parse_deps_toml("[dependencies]\nferal_ordering_core = \"0.2.1\"\n")
        .expect_err("syntactically valid names must still be explicitly allowlisted");
    assert!(!err.0.contains("invalid crate name"));
    assert!(err
        .0
        .contains("`feral_ordering_core` is not in the reviewed direct-dependency allowlist"));
}

#[test]
fn quoted_crate_names_are_rejected_with_actionable_errors() {
    for name in ["\"serde\"", "'serde'"] {
        let src = format!("[dependencies]\n{name} = \"1.0.0\"\n");
        let err = parse_deps_toml(&src).expect_err("quoted crate name should be rejected");
        assert!(err.0.contains("deps.toml:2: invalid crate name"));
        assert!(err.0.contains("ASCII letters"));
        assert!(err.0.contains("hyphens (`-`)"));
        assert!(err.0.contains("underscores (`_`)"));
    }
}

#[test]
fn whitespace_and_control_characters_in_crate_names_are_rejected() {
    for name in ["bad name", "bad\tname", "bad\u{b}name", "bad\0name"] {
        let src = format!("[dependencies]\n{name} = \"1.0.0\"\n");
        let err =
            parse_deps_toml(&src).expect_err("whitespace/control character should be rejected");
        assert!(err.0.contains("deps.toml:2: invalid crate name"));
    }
}

#[test]
fn separator_characters_in_crate_names_are_rejected() {
    for name in ["bad.name", "bad/name", "bad\\name", "bad:name", "bad@name"] {
        let src = format!("[dependencies]\n{name} = \"1.0.0\"\n");
        let err = parse_deps_toml(&src).expect_err("separator should be rejected");
        assert!(err.0.contains("deps.toml:2: invalid crate name"));
    }
}

#[test]
fn empty_file_is_ok_and_empty() {
    assert!(parse_deps_toml("").unwrap().is_empty());
    assert!(parse_deps_toml("[dependencies]\n").unwrap().is_empty());
}

#[test]
fn inline_table_is_rejected() {
    // The form that could carry git/path/features — must be impossible.
    let src = "[dependencies]\nevil = { git = \"https://x/y\" }\n";
    assert!(parse_deps_toml(src).is_err());
}

#[test]
fn unknown_section_is_rejected() {
    let src = "[build-dependencies]\ncc = \"1\"\n";
    assert!(parse_deps_toml(src).is_err());
}

#[test]
fn non_semverish_version_is_rejected() {
    // A version string must look like digits/dots (no "*", no ranges, no git refs).
    let src = "[dependencies]\nrand = \"*\"\n";
    assert!(parse_deps_toml(src).is_err());
}

fn vwrite(dir: &std::path::Path, rel: &str, body: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

#[test]
fn clean_vendor_tree_passes() {
    let root = std::env::temp_dir().join("ssi-vendor-clean");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(&root, "rand-0.8.5/Cargo.toml", "[package]\nname=\"rand\"\n");
    vwrite(&root, "rand-0.8.5/src/lib.rs", "pub fn f() -> u32 { 1 }\n");
    assert!(scan_vendored_tree(&root).is_ok());
}

#[test]
fn default_build_script_is_rejected() {
    let root = std::env::temp_dir().join("ssi-vendor-build-rs");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "untrusted-1.0/Cargo.toml",
        "[package]\nname=\"untrusted\"\nversion=\"1.0.0\"\n",
    );
    vwrite(
        &root,
        "untrusted-1.0/build.rs",
        "fn main() { panic!(\"ran\"); }\n",
    );
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn custom_build_script_is_rejected() {
    let root = std::env::temp_dir().join("ssi-vendor-custom-build");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "untrusted-1.0/Cargo.toml",
        "[package]\nname=\"untrusted\"\nversion=\"1.0.0\"\nbuild=\"scripts/generate.rs\"\n",
    );
    vwrite(&root, "untrusted-1.0/scripts/generate.rs", "fn main() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn explicitly_disabled_build_script_is_allowed() {
    let root = std::env::temp_dir().join("ssi-vendor-disabled-build");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "plain-1.0/Cargo.toml",
        "[package]\nname=\"plain\"\nversion=\"1.0.0\"\nbuild=false\n",
    );
    // Cargo ignores this file because the package explicitly disables build
    // script discovery; retaining it avoids rejecting normalized crates that
    // ship inert build.rs files.
    vwrite(
        &root,
        "plain-1.0/build.rs",
        "fn main() { panic!(\"inert\"); }\n",
    );
    assert!(scan_vendored_tree(&root).is_ok());
}

#[test]
fn proc_macro_crate_is_rejected() {
    let root = std::env::temp_dir().join("ssi-vendor-proc-macro");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "untrusted-derive-1.0/Cargo.toml",
        "[package]\nname=\"untrusted-derive\"\nversion=\"1.0.0\"\nbuild=false\n\
         [lib]\nproc-macro=true\n",
    );
    vwrite(
        &root,
        "untrusted-derive-1.0/src/lib.rs",
        "pub fn derive() {}\n",
    );
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn proc_macro_crate_type_is_rejected() {
    let root = std::env::temp_dir().join("ssi-vendor-proc-macro-type");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "untrusted-derive-1.0/Cargo.toml",
        "[package]\nname=\"untrusted-derive\"\nversion=\"1.0.0\"\nbuild=false\n\
         [lib]\ncrate-type=[\"proc-macro\"]\n",
    );
    vwrite(
        &root,
        "untrusted-derive-1.0/src/lib.rs",
        "pub fn derive() {}\n",
    );
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn only_exact_pinned_harness_proc_macro_is_exempt() {
    let root = std::env::temp_dir().join("ssi-vendor-pinned-proc-macro");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "serde_derive/Cargo.toml",
        "[package]\nname=\"serde_derive\"\nversion=\"1.0.228\"\nbuild=false\n\
         [lib]\nproc-macro=true\n",
    );
    vwrite(&root, "serde_derive/src/lib.rs", "pub fn derive() {}\n");
    assert!(scan_vendored_tree(&root).is_ok());

    vwrite(
        &root,
        "serde_derive/Cargo.toml",
        "[package]\nname=\"serde_derive\"\nversion=\"1.0.229\"\nbuild=false\n\
         [lib]\nproc-macro=true\n",
    );
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn cc_build_dependency_is_rejected() {
    // A `cc`/`cmake`/`bindgen`/... build-dependency compiles native code from
    // build.rs — a sound, false-positive-free native signal we reject.
    let root = std::env::temp_dir().join("ssi-vendor-cc");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "flate2-1.0/Cargo.toml",
        "[package]\nname=\"flate2\"\n[build-dependencies]\ncc = \"1.0\"\n",
    );
    vwrite(&root, "flate2-1.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn cc_build_dependency_table_form_is_rejected() {
    // The `[build-dependencies.cc]` table form must be caught too.
    let root = std::env::temp_dir().join("ssi-vendor-cc-table");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "foo-1.0/Cargo.toml",
        "[package]\nname=\"foo\"\n[build-dependencies.cc]\nversion = \"1.0\"\n",
    );
    vwrite(&root, "foo-1.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn target_conditional_cc_build_dependency_is_rejected() {
    // A C-toolchain build-dep under a target-conditional section
    // (`[target.'cfg(...)'.build-dependencies]`) must be caught too — otherwise
    // a crate that compiles C only on some target would pass the scan on a host
    // where that target's section is inert.
    let root = std::env::temp_dir().join("ssi-vendor-cc-target");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "foo-1.0/Cargo.toml",
        "[package]\nname=\"foo\"\n[target.'cfg(unix)'.build-dependencies]\ncc = \"1.0\"\n",
    );
    vwrite(&root, "foo-1.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn workspace_inherited_cc_build_dependency_is_rejected() {
    // The dotted `cc.workspace = true` / `cc.version = "1"` key forms must be
    // recognized as the `cc` build-dep, not read as a key named `cc.workspace`.
    let root = std::env::temp_dir().join("ssi-vendor-cc-workspace");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "foo-1.0/Cargo.toml",
        "[package]\nname=\"foo\"\n[build-dependencies]\ncc.workspace = true\n",
    );
    vwrite(&root, "foo-1.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn renamed_cc_build_dependency_inline_is_rejected() {
    // A C-toolchain crate renamed via `package = "cc"` must not evade the scan
    // by hiding under a benign key (inline-table form).
    let root = std::env::temp_dir().join("ssi-vendor-cc-rename-inline");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "foo-1.0/Cargo.toml",
        "[package]\nname=\"foo\"\n[build-dependencies]\nmycc = { package = \"cc\", version = \"1.0\" }\n",
    );
    vwrite(&root, "foo-1.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn renamed_cc_build_dependency_table_header_is_rejected() {
    // The renamed form as a `package = "cc"` line under a
    // `[build-dependencies.<name>]` table header must also be caught.
    let root = std::env::temp_dir().join("ssi-vendor-cc-rename-table");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "foo-1.0/Cargo.toml",
        "[package]\nname=\"foo\"\n[build-dependencies.mycc]\npackage = \"cc\"\nversion = \"1.0\"\n",
    );
    vwrite(&root, "foo-1.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn prebuilt_native_artifact_is_rejected() {
    // A committed linkable blob bypasses building from source.
    let root = std::env::temp_dir().join("ssi-vendor-blob");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(&root, "foo-1.0/Cargo.toml", "[package]\nname=\"foo\"\n");
    vwrite(&root, "foo-1.0/src/lib.rs", "pub fn f() {}\n");
    vwrite(&root, "foo-1.0/vendor/libfoo.a", "\x7fELF not really\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn bare_links_version_guard_is_allowed() {
    // `links` alone links NOTHING in the common single-version-guard idiom
    // (e.g. rayon-core, which feral's tree pulls in). Rejecting it would bar a
    // pure-Rust crate; the no-C-compiler build backstops any real native link.
    let root = std::env::temp_dir().join("ssi-vendor-links-guard");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "rayon-core-1.13.0/Cargo.toml",
        "[package]\nname=\"rayon-core\"\nlinks=\"rayon-core\"\n",
    );
    vwrite(&root, "rayon-core-1.13.0/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_ok());
}

#[test]
fn c_abi_export_and_proc_macro_api_reference_in_dep_are_allowed() {
    // A dependency may legitimately EXPORT a C ABI (`#[no_mangle] extern "C" fn`,
    // a definition — no foreign code runs) or refer to the proc_macro API from
    // ordinary source. Source tokens are not enough to classify a crate; the
    // manifest's `proc-macro = true` target declaration is what the gate rejects.
    let root = std::env::temp_dir().join("ssi-vendor-cabi-export");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "feralish-0.1.0/Cargo.toml",
        "[package]\nname=\"feralish\"\n",
    );
    vwrite(
        &root,
        "feralish-0.1.0/src/lib.rs",
        "#[no_mangle]\npub extern \"C\" fn exported() -> u32 { 1 }\nuse proc_macro::TokenStream;\n",
    );
    assert!(scan_vendored_tree(&root).is_ok());
}

#[test]
fn sys_suffix_crate_is_rejected() {
    let root = std::env::temp_dir().join("ssi-vendor-sys");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "openssl-sys-0.9/Cargo.toml",
        "[package]\nname=\"openssl-sys\"\n",
    );
    vwrite(&root, "openssl-sys-0.9/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn sys_suffix_crate_with_hyphenated_prerelease_version_is_rejected() {
    // A hyphenated pre-release version (`1.0.0-alpha.1`) must not let a `*-sys`
    // crate slip past the name check: a naive last-hyphen split would leave
    // `foo-sys-1.0.0`, which does not end in `-sys`. The version boundary is the
    // last `-` before a digit, so the name is correctly `foo-sys`.
    let root = std::env::temp_dir().join("ssi-vendor-sys-prerelease");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "foo-sys-1.0.0-alpha.1/Cargo.toml",
        "[package]\nname=\"foo-sys\"\n",
    );
    vwrite(&root, "foo-sys-1.0.0-alpha.1/src/lib.rs", "pub fn f() {}\n");
    assert!(scan_vendored_tree(&root).is_err());
}

#[test]
fn non_sys_crate_with_hyphenated_name_passes() {
    // An innocent hyphenated crate name whose last component isn't `sys` must not
    // be falsely rejected (guards against over-eager name stripping).
    let root = std::env::temp_dir().join("ssi-vendor-hyphen-ok");
    let _ = std::fs::remove_dir_all(&root);
    vwrite(
        &root,
        "system-deps-1.2.3/Cargo.toml",
        "[package]\nname=\"system-deps\"\n",
    );
    vwrite(
        &root,
        "system-deps-1.2.3/src/lib.rs",
        "pub fn f() -> u32 { 1 }\n",
    );
    assert!(scan_vendored_tree(&root).is_ok());
}
