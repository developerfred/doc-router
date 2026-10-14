use crate::{parser, router, scanner};
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Task {
    id: String,
    description: String,
}

#[derive(Debug, serde::Serialize)]
struct TaskResult {
    id: String,
    description: String,
    decision: String,
    navigated_tokens: usize,
    total_tokens: usize,
    savings_pct: f64,
    sections_selected: usize,
}

pub fn run(corpus: &Path, tasks_file: &Path, format: &str) -> Result<()> {
    let raw = std::fs::read_to_string(tasks_file)?;
    let tasks: Vec<Task> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()?;

    let docs = scanner::scan_docs(corpus)?;
    let tree = parser::build_tree(&docs)?;

    let mut results = Vec::new();
    for t in &tasks {
        let r = router::route(&t.description, &tree)?;
        let pct = if r.total_document_tokens > 0 {
            (1.0 - (r.total_navigated_tokens as f64 / r.total_document_tokens as f64)) * 100.0
        } else {
            0.0
        };
        results.push(TaskResult {
            id: t.id.clone(),
            description: t.description.clone(),
            decision: format!("{:?}", r.decision).to_lowercase(),
            navigated_tokens: r.total_navigated_tokens,
            total_tokens: r.total_document_tokens,
            savings_pct: pct,
            sections_selected: r.navigate.len(),
        });
    }

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&results)?),
        _ => print_table(&results),
    }
    Ok(())
}

fn print_table(results: &[TaskResult]) {
    let mut s: Vec<f64> = results.iter().map(|r| r.savings_pct).collect();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median = if s.is_empty() {
        0.0
    } else if s.len() % 2 == 0 {
        (s[s.len() / 2 - 1] + s[s.len() / 2]) / 2.0
    } else {
        s[s.len() / 2]
    };

    let skips = results.iter().filter(|r| r.decision == "skip").count();
    let reads = results.iter().filter(|r| r.decision == "read").count();
    let unc = results.iter().filter(|r| r.decision == "uncertain").count();

    println!();
    println!("===========================================================");
    println!("  doc-router benchmark");
    println!("===========================================================");
    println!("  tasks run:      {}", results.len());
    println!(
        "  total tokens:   {}",
        results.first().map(|r| r.total_tokens).unwrap_or(0)
    );
    println!("  median savings: {median:.1}%");
    println!("  decisions:      read={reads} uncertain={unc} skip={skips}");
    println!("===========================================================");
    println!();
    println!(
        "{:<6} {:<40} {:>10} {:>8} {:>7}",
        "ID", "Task", "Decision", "Tokens", "Sav%"
    );
    println!("{}", "-".repeat(76));
    for r in results {
        let short: String = r.description.chars().take(38).collect();
        println!(
            "{:<6} {:<40} {:>10} {:>8} {:>6.1}%",
            r.id, short, r.decision, r.navigated_tokens, r.savings_pct
        );
    }
}
