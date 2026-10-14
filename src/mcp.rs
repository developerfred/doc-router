use crate::{expand, parser, router, scanner};
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Req {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

pub fn serve(root: &Path) -> Result<()> {
    let root: PathBuf = root.to_path_buf();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    eprintln!("[doc-router-mcp] ready, root={}", root.display());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let req: Req = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[mcp] parse error: {e}");
                continue;
            }
        };

        if req.id.is_none() && req.method.starts_with("notifications/") {
            continue;
        }

        let id = req.id.clone().unwrap_or(Value::Null);
        let result = handle(&root, &req);

        let resp = match result {
            Ok(v) => json!({"jsonrpc": "2.0", "id": id, "result": v}),
            Err(e) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32603, "message": e}}),
        };

        writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle(root: &Path, req: &Req) -> std::result::Result<Value, String> {
    match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "doc-router", "version": env!("CARGO_PKG_VERSION") }
        })),
        "tools/list" => Ok(json!({
            "tools": [
                {
                    "name": "doc_route",
                    "description": "Given a task description, return the minimal set of documentation sections to read.",
                    "inputSchema": {
                        "type": "object",
                        "properties": { "task": { "type": "string" } },
                        "required": ["task"]
                    }
                },
                {
                    "name": "doc_expand",
                    "description": "Expand a section_id into raw Markdown text.",
                    "inputSchema": {
                        "type": "object",
                        "properties": { "section_id": { "type": "string" } },
                        "required": ["section_id"]
                    }
                }
            ]
        })),
        "tools/call" => call_tool(root, &req.params),
        other => Err(format!("unknown method: {other}")),
    }
}

fn call_tool(root: &Path, params: &Value) -> std::result::Result<Value, String> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("missing tool name")?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
        "doc_route" => {
            let task = args
                .get("task")
                .and_then(|v| v.as_str())
                .ok_or("missing task")?;
            let docs = scanner::scan_docs(root).map_err(|e| e.to_string())?;
            let tree = parser::build_tree(&docs).map_err(|e| e.to_string())?;
            let r = router::route(task, &tree).map_err(|e| e.to_string())?;
            let text = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?;
            Ok(json!({"content": [{"type": "text", "text": text}]}))
        }
        "doc_expand" => {
            let id = args
                .get("section_id")
                .and_then(|v| v.as_str())
                .ok_or("missing section_id")?;
            let text = expand::expand_section(root, id).map_err(|e| e.to_string())?;
            Ok(json!({"content": [{"type": "text", "text": text}]}))
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}
