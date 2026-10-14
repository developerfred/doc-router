use crate::model::{ParsedDoc, Section};
use crate::scanner::DocFile;
use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};

fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

pub fn build_tree(docs: &[DocFile]) -> Result<Vec<ParsedDoc>> {
    let mut out = Vec::new();
    for doc in docs {
        let sections = parse_sections(&doc.path, &doc.content)?;
        out.push(ParsedDoc {
            path: doc.path.clone(),
            total_tokens: estimate_tokens(&doc.content),
            sections,
        });
    }
    Ok(out)
}

struct Raw {
    title: String,
    level: u8,
    line_start: usize,
    byte_start: usize,
}

fn parse_sections(path: &str, content: &str) -> Result<Vec<Section>> {
    let mut line_starts = vec![0usize];
    for (i, b) in content.bytes().enumerate() {
        if b == b'\n' {
            line_starts.push(i + 1);
        }
    }
    let line_of = |off: usize| -> usize {
        match line_starts.binary_search(&off) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    };

    let parser = Parser::new_ext(content, Options::all()).into_offset_iter();
    let mut raw: Vec<Raw> = Vec::new();

    for (event, range) in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                raw.push(Raw {
                    title: String::new(),
                    level: lvl(level),
                    line_start: line_of(range.start),
                    byte_start: range.start,
                });
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some(r) = raw.last_mut() {
                    r.title.push_str(&t);
                }
            }
            _ => {}
        }
    }

    let mut sections = Vec::new();
    for (i, r) in raw.iter().enumerate() {
        let byte_end = if i + 1 < raw.len() {
            raw[i + 1].byte_start.saturating_sub(1)
        } else {
            content.len()
        };
        let line_end = if i + 1 < raw.len() {
            raw[i + 1].line_start.saturating_sub(1)
        } else {
            content.lines().count()
        };
        let title = r.title.trim().to_string();
        let id = format!("{}::{}::{}", path, r.level, slug(&title));
        let text = &content[r.byte_start.min(content.len())..byte_end.min(content.len())];

        sections.push(Section {
            id,
            doc_path: path.to_string(),
            title,
            level: r.level,
            line_start: r.line_start,
            line_end,
            byte_start: r.byte_start,
            byte_end,
            token_estimate: estimate_tokens(text),
        });
    }
    Ok(sections)
}

fn lvl(l: HeadingLevel) -> u8 {
    match l {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn slug(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
