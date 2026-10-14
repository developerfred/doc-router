use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use crate::model::{ParsedDoc, Section};
use crate::scanner::DocFile;

fn estimate_tokens(text: &str) -> usize {
    // ~4 chars per token for English/prose; conservative.
    (text.len() / 4).max(1)
}

pub fn build_tree(docs: &[DocFile]) -> Result<Vec<ParsedDoc>> {
    let mut parsed = Vec::new();
    for doc in docs {
        let sections = parse_sections(&doc.path, &doc.content)?;
        parsed.push(ParsedDoc {
            path: doc.path.clone(),
            total_tokens: estimate_tokens(&doc.content),
            sections,
        });
    }
    Ok(parsed)
}

struct SectionBuilder {
    title: String,
    level: u8,
    line_start: usize,
    byte_start: usize,
}

fn parse_sections(path: &str, content: &str) -> Result<Vec<Section>> {
    let line_starts = compute_line_starts(content);
    let line_of = |offset: usize| -> usize {
        match line_starts.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    };

    let parser = Parser::new_ext(content, Options::all()).into_offset_iter();
    let mut raw: Vec<SectionBuilder> = Vec::new();

    for (event, range) in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                raw.push(SectionBuilder {
                    title: String::new(),
                    level: heading_level_to_u8(level),
                    line_start: line_of(range.start),
                    byte_start: range.start,
                });
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(b) = raw.last_mut() {
                    b.title.push_str(&text);
                }
            }
            _ => {}
        }
    }

    let mut sections = Vec::new();
    for (i, builder) in raw.iter().enumerate() {
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

        let title = builder.title.trim().to_string();
        let id = format!("{}::{}::{}", path, builder.level, slugify(&title));
        let text = &content[builder.byte_start.min(content.len())..byte_end.min(content.len())];

        sections.push(Section {
            id,
            doc_path: path.to_string(),
            title,
            level: builder.level,
            line_start: builder.line_start,
            line_end,
            byte_start: builder.byte_start,
            byte_end,
            token_estimate: estimate_tokens(text),
        });
    }

    Ok(sections)
}

fn compute_line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in content.bytes().enumerate() {
        if b == b'\n' { starts.push(i + 1); }
    }
    starts
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
