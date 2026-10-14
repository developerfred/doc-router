use crate::model::{Decision, Navigation, ParsedDoc, RouteResult};
use anyhow::Result;

const TOKEN_BUDGET: usize = 800;
const READ_THRESHOLD: f32 = 5.0;
const SKIP_THRESHOLD: f32 = 6.0;

pub fn route(task: &str, docs: &[ParsedDoc]) -> Result<RouteResult> {
    let task_lower = task.to_lowercase();
    let tokens: Vec<&str> = task_lower
        .split_whitespace()
        .filter(|t| t.len() > 2 && !is_stopword(t))
        .collect();

    let total_doc_tokens: usize = docs.iter().map(|d| d.total_tokens).sum();
    let mut scored: Vec<(f32, &ParsedDoc, &crate::model::Section)> = Vec::new();

    for doc in docs {
        let bonus = doc_bonus(&doc.path);
        for section in &doc.sections {
            let title_lower = section.title.to_lowercase();
            let mut score = bonus;
            for t in &tokens {
                if title_lower.contains(t) {
                    score += 5.0;
                }
            }
            if score > 0.0 {
                scored.push((score, doc, section));
            }
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let best = scored.first().map(|(s, _, _)| *s).unwrap_or(0.0);

    let decision = if best < SKIP_THRESHOLD {
        Decision::Skip
    } else if best < READ_THRESHOLD {
        Decision::Uncertain
    } else {
        Decision::Read
    };

    let mut navigate = Vec::new();
    let mut total = 0usize;

    if decision != Decision::Skip {
        for (score, doc, section) in scored {
            if total + section.token_estimate > TOKEN_BUDGET {
                continue;
            }
            let reason = if score >= READ_THRESHOLD {
                "direct title match for task keywords".to_string()
            } else if score >= 2.0 {
                "priority documentation file".to_string()
            } else {
                "partial keyword match".to_string()
            };
            navigate.push(Navigation {
                doc: doc.path.clone(),
                section_id: section.id.clone(),
                title: section.title.clone(),
                reason,
                score,
                token_estimate: section.token_estimate,
            });
            total += section.token_estimate;
        }
    }

    let savings = if total_doc_tokens > 0 {
        format!(
            "{:.1}%",
            (1.0 - (total as f64 / total_doc_tokens as f64)) * 100.0
        )
    } else {
        "0.0%".to_string()
    };

    Ok(RouteResult {
        task: task.to_string(),
        decision,
        navigate,
        total_navigated_tokens: total,
        total_document_tokens: total_doc_tokens,
        savings_ratio: savings,
    })
}

fn doc_bonus(path: &str) -> f32 {
    let p = std::path::Path::new(path);
    let depth = p.components().count();
    if depth > 3 {
        return 0.0;
    }
    if path.ends_with("/AGENTS.md") || path.ends_with("/CLAUDE.md") {
        2.0
    } else if path.ends_with("/README.md") {
        1.0
    } else {
        0.0
    }
}

fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        "the" | "and" | "for" | "with" | "that" | "this" | "from" | "into" | "add"
    )
}

pub fn print_human(r: &RouteResult) {
    println!("Task: {}", r.task);
    println!("Decision: {:?}", r.decision);
    println!("Total document tokens: {}", r.total_document_tokens);
    println!("Navigated tokens: {}", r.total_navigated_tokens);
    println!("Savings: {}", r.savings_ratio);
    println!();
    if r.navigate.is_empty() {
        println!("   (nothing to read - proceed directly)");
    } else {
        for n in &r.navigate {
            println!("  -> {} [{}]", n.doc, n.title);
            println!("     reason: {}", n.reason);
            println!("     score: {:.2} | ~{} tokens", n.score, n.token_estimate);
        }
    }
}
