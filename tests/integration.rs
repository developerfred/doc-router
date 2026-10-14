use std::fs;
use tempfile::TempDir;

// ─── helpers ───────────────────────────────────────────────

fn write_doc(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let p = dir.path().join(name);
    fs::write(&p, content).unwrap();
    p
}

// ─── parser ────────────────────────────────────────────────

#[test]
fn parser_extracts_sections_by_heading() {
    let dir = TempDir::new().unwrap();
    write_doc(
        &dir,
        "AGENTS.md",
        r#"# Project

Intro text.

## Commands

Run tests.

## Architecture

Layered.
"#,
    );

    let docs = doc_router_lib::scanner::scan_docs(dir.path()).unwrap();
    let tree = doc_router_lib::parser::build_tree(&docs).unwrap();

    assert_eq!(tree.len(), 1);
    let sections = &tree[0].sections;
    assert_eq!(sections.len(), 3, "expected H1 + 2 H2");

    assert_eq!(sections[0].title, "Project");
    assert_eq!(sections[0].level, 1);

    assert_eq!(sections[1].title, "Commands");
    assert_eq!(sections[1].level, 2);

    assert_eq!(sections[2].title, "Architecture");
    assert_eq!(sections[2].level, 2);
}

#[test]
fn parser_computes_byte_offsets() {
    let dir = TempDir::new().unwrap();
    write_doc(&dir, "X.md", "## A\n\nfoo\n\n## B\n\nbar\n");

    let docs = doc_router_lib::scanner::scan_docs(dir.path()).unwrap();
    let tree = doc_router_lib::parser::build_tree(&docs).unwrap();
    let s = &tree[0].sections;

    // primeira seção deve conter "foo", não "bar"
    let content = fs::read_to_string(dir.path().join("X.md")).unwrap();
    let text_a = &content[s[0].byte_start..s[0].byte_end];
    assert!(text_a.contains("foo"));
    assert!(!text_a.contains("bar"));
}

// ─── router ────────────────────────────────────────────────

#[test]
fn router_skips_when_no_match() {
    let dir = TempDir::new().unwrap();
    write_doc(&dir, "README.md", "## Installation\n\nrun cargo\n");

    let docs = doc_router_lib::scanner::scan_docs(dir.path()).unwrap();
    let tree = doc_router_lib::parser::build_tree(&docs).unwrap();

    let r = doc_router_lib::router::route("quantum flux capacitor calibration", &tree).unwrap();

    assert_eq!(format!("{:?}", r.decision), "Skip");
    assert_eq!(r.navigate.len(), 0);
    assert_eq!(r.savings_ratio, "100.0%");
}

#[test]
fn router_reads_when_title_matches() {
    let dir = TempDir::new().unwrap();
    write_doc(
        &dir,
        "AGENTS.md",
        r#"# Conventions

## Rate limiting

All endpoints need rate limiting.
"#,
    );

    let docs = doc_router_lib::scanner::scan_docs(dir.path()).unwrap();
    let tree = doc_router_lib::parser::build_tree(&docs).unwrap();

    let r = doc_router_lib::router::route("add rate limiting to payments", &tree).unwrap();

    assert_eq!(format!("{:?}", r.decision), "Read");
    assert!(r.navigate.iter().any(|n| n.title == "Rate limiting"));
}

// ─── expand ────────────────────────────────────────────────

#[test]
fn expand_returns_exact_section_text() {
    let dir = TempDir::new().unwrap();
    write_doc(
        &dir,
        "A.md",
        "## First\n\nfoo content\n\n## Second\n\nbar content\n",
    );

    let docs = doc_router_lib::scanner::scan_docs(dir.path()).unwrap();
    let tree = doc_router_lib::parser::build_tree(&docs).unwrap();
    let section_id = &tree[0].sections[0].id;

    let text = doc_router_lib::expand::expand_section(dir.path(), section_id).unwrap();
    assert!(text.contains("foo content"));
    assert!(!text.contains("bar content"));
}
