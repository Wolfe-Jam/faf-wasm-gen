/*!
 * faf-wasm-gen
 *
 * Rust WASM Generator for FAF (Foundational AI-context Format)
 *
 * SPEC: faf-cli v6.8 — faf_version "3.3", 33-slot Mk4 model
 * BIBLE: /Users/wolfejam/FAF/cli (faf-cli) — src/core/slots.ts
 *
 * MISSION: Generate project.faf matching faf-cli v6.8 output:
 *   - faf_version "3.3"
 *   - 33 Mk4 canonical slots (21 base + 12 enterprise), `slotignored` for
 *     slots outside the detected app-type's active categories
 *   - Lean section set (project / instant_context / stack / human_context /
 *     tags / state / metadata / monorepo). NO embedded score — faf-cli
 *     computes score = populated/active on READ (score is not stored).
 *
 * Ported 2026-06-03 from the obsolete v4.2.1 / faf_version 2.5.0 generator.
 */

use wasm_bindgen::prelude::*;
use serde_json::Value;
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;

/// Mk4 slot categories (see slots.ts).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Cat {
    Project,
    Human,
    Frontend,
    Backend,
    Universal,
    EntInfra,
    EntApp,
    EntOps,
}

/// app_type → active categories. Mirrors APP_TYPE_CATEGORIES in slots.ts.
/// Slots whose category is NOT in this set render as `slotignored`.
fn active_cats(app_type: &str) -> Vec<Cat> {
    use Cat::*;
    match app_type {
        "documentation" | "encyclopedia" => vec![Project, Human],
        "cli" | "library" | "sdk" | "wasm" | "html" => vec![Project, Human, Universal],
        "frontend" | "website" | "mobile" | "extension" => {
            vec![Project, Frontend, Human, Universal]
        }
        "mcp" | "backend" | "data-science" => vec![Project, Backend, Human, Universal],
        "fullstack" | "svelte" | "framework" => {
            vec![Project, Frontend, Backend, Universal, Human]
        }
        "monorepo-root" => vec![Project, Human, EntInfra, EntApp, EntOps],
        "saas" => vec![Project, Frontend, Backend, Universal, Human, EntApp],
        "mcpaas" => vec![Project, Backend, Universal, Human, EntApp, EntOps],
        "enterprise" => vec![
            Project, Frontend, Backend, Universal, Human, EntInfra, EntApp, EntOps,
        ],
        // Default to library shape (project + human + universal).
        _ => vec![Project, Human, Universal],
    }
}

/// The 19 slots that live under the `stack:` section (on-wire key, category).
/// Order matches slots.ts indices 10-24, 27-30.
const STACK_SLOTS: &[(&str, Cat)] = &[
    ("frontend", Cat::Frontend),
    ("css_framework", Cat::Frontend),
    ("ui_library", Cat::Frontend),
    ("state_management", Cat::Frontend),
    ("backend", Cat::Backend),
    ("api_type", Cat::Backend),
    ("runtime", Cat::Backend),
    ("database", Cat::Backend),
    ("connection", Cat::Backend),
    ("hosting", Cat::Universal),
    ("build", Cat::Universal),
    ("cicd", Cat::Universal),
    ("monorepo_tool", Cat::EntInfra),
    ("package_manager", Cat::EntInfra),
    ("workspaces", Cat::EntInfra),
    ("admin", Cat::EntApp),
    ("cache", Cat::EntApp),
    ("search", Cat::EntApp),
    ("storage", Cat::EntApp),
];

/// The 5 slots that live under the `monorepo:` section (slots.ts 25,26,31,32,33).
const MONOREPO_SLOTS: &[(&str, Cat)] = &[
    ("packages_count", Cat::EntInfra),
    ("build_orchestrator", Cat::EntInfra),
    ("versioning_strategy", Cat::EntOps),
    ("shared_configs", Cat::EntOps),
    ("remote_cache", Cat::EntOps),
];

#[derive(Debug, Default)]
struct HumanContext {
    who: String,
    what: String,
    why: String,
    where_: String,
    when: String,
    how: String,
}

/// Generate project.faf (faf-cli v6.8 / faf_version "3.3") from repo metadata.
///
/// # Parameters
/// - `repo_name`: Repository name (e.g., "grok-1")
/// - `owner`: Repository owner (e.g., "xai-org")
/// - `description`: Optional repository description
/// - `readme`: Optional README.md content
/// - `dependency_file`: Optional dependency file (package.json, pyproject.toml, Cargo.toml)
/// - `language`: Optional primary language from the GitHub API
///
/// # Returns
/// project.faf YAML (no embedded score — faf-cli computes it on read).
#[wasm_bindgen]
pub fn generate_faf(
    repo_name: String,
    owner: String,
    description: Option<String>,
    readme: Option<String>,
    dependency_file: Option<String>,
    language: Option<String>,
) -> Result<String, JsValue> {
    let app_type = detect_app_type(
        readme.as_deref(),
        dependency_file.as_deref(),
        language.as_deref(),
    );
    let stack = detect_stack(dependency_file.as_deref(), language.as_deref());
    let human = extract_human_context(
        readme.as_deref(),
        &repo_name,
        &owner,
        description.as_deref(),
    );
    let version = detect_version(dependency_file.as_deref());
    let framework = stack.get("frontend").or_else(|| stack.get("backend")).cloned();
    let primary = primary_stack(&stack, language.as_deref());

    Ok(generate_yaml(
        &repo_name, &owner, &app_type, &stack, &human,
        language.as_deref(), &primary, version.as_deref(), framework.as_deref(),
    ))
}

/// Detect the canonical app_type (slots.ts ladder). Highest signal wins.
fn detect_app_type(
    readme: Option<&str>,
    dependency_file: Option<&str>,
    language: Option<&str>,
) -> String {
    let readme_lower = readme.map(|s| s.to_lowercase()).unwrap_or_default();
    let dep_lower = dependency_file.map(|s| s.to_lowercase()).unwrap_or_default();

    // MCP server (rmcp / @modelcontextprotocol / fastmcp)
    if dep_lower.contains("rmcp")
        || dep_lower.contains("modelcontextprotocol")
        || dep_lower.contains("fastmcp")
        || dep_lower.contains("\"mcp\"")
    {
        return "mcp".to_string();
    }

    // Data science / ML
    if dep_lower.contains("jax")
        || dep_lower.contains("pytorch")
        || dep_lower.contains("tensorflow")
        || dep_lower.contains("torch")
        || dep_lower.contains("flax")
        || dep_lower.contains("transformers")
        || readme_lower.contains("machine learning")
        || readme_lower.contains("deep learning")
        || readme_lower.contains("neural network")
    {
        return "data-science".to_string();
    }

    // Frontend frameworks
    if dep_lower.contains("\"react\"")
        || dep_lower.contains("\"vue\"")
        || dep_lower.contains("\"svelte\"")
        || dep_lower.contains("@angular/core")
        || dep_lower.contains("\"next\"")
    {
        return "frontend".to_string();
    }

    // Backend frameworks
    if dep_lower.contains("\"express\"")
        || dep_lower.contains("\"fastapi\"")
        || dep_lower.contains("\"django\"")
        || dep_lower.contains("\"flask\"")
        || dep_lower.contains("\"axum\"")
        || dep_lower.contains("\"actix")
    {
        return "backend".to_string();
    }

    // CLI tools
    if dep_lower.contains("\"commander\"")
        || dep_lower.contains("\"yargs\"")
        || dep_lower.contains("\"clap\"")
        || dep_lower.contains("\"click\"")
        || dep_lower.contains("\"argparse\"")
        || dep_lower.contains("\"typer\"")
    {
        return "cli".to_string();
    }

    // SDK by name convention (public dev kit)
    if let Some(l) = language {
        if l.eq_ignore_ascii_case("rust") {
            return "library".to_string();
        }
    }

    "library".to_string()
}

/// Detect stack slot values from a dependency file. Keys are on-wire slot names.
fn detect_stack(dependency_file: Option<&str>, language: Option<&str>) -> HashMap<String, String> {
    let mut s: HashMap<String, String> = HashMap::new();
    let dep = match dependency_file {
        Some(d) => d,
        None => return s,
    };
    let dep_lower = dep.to_lowercase();

    // Python (pyproject.toml)
    if dep.contains("[tool.poetry]") || dep.contains("[project]") && dep_lower.contains("python") {
        s.insert("runtime".into(), "Python".into());
        if dep.contains("[tool.poetry]") {
            s.insert("package_manager".into(), "poetry".into());
            s.insert("build".into(), "poetry".into());
        }
        if dep_lower.contains("fastapi") {
            s.insert("backend".into(), "FastAPI".into());
            s.insert("api_type".into(), "REST".into());
        } else if dep_lower.contains("django") {
            s.insert("backend".into(), "Django".into());
        } else if dep_lower.contains("flask") {
            s.insert("backend".into(), "Flask".into());
        }
        if dep_lower.contains("sqlalchemy") || dep_lower.contains("django.db") {
            s.insert("database".into(), "SQL".into());
        }
    }

    // JS/TS (package.json)
    if let Ok(pkg) = serde_json::from_str::<Value>(dep) {
        if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
            if deps.contains_key("react") {
                s.insert("frontend".into(), "React".into());
            } else if deps.contains_key("vue") {
                s.insert("frontend".into(), "Vue".into());
            } else if deps.contains_key("svelte") {
                s.insert("frontend".into(), "Svelte".into());
            } else if deps.contains_key("@angular/core") {
                s.insert("frontend".into(), "Angular".into());
            }
            if deps.contains_key("tailwindcss") {
                s.insert("css_framework".into(), "Tailwind".into());
            }
            if deps.contains_key("express") {
                s.insert("backend".into(), "Express".into());
                s.insert("api_type".into(), "REST".into());
            } else if deps.contains_key("fastify") {
                s.insert("backend".into(), "Fastify".into());
                s.insert("api_type".into(), "REST".into());
            }
            s.insert("runtime".into(), "Node.js".into());
            s.insert("package_manager".into(), "npm".into());
        }
        if let Some(dev) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
            if dev.contains_key("vite") {
                s.insert("build".into(), "Vite".into());
            } else if dev.contains_key("webpack") {
                s.insert("build".into(), "Webpack".into());
            }
        }
    }

    // Rust (Cargo.toml)
    if dep.contains("[package]") && (dep_lower.contains("edition") || dep_lower.contains("cargo")) {
        s.insert("package_manager".into(), "cargo".into());
        s.insert("runtime".into(), "Rust".into());
        s.insert("build".into(), "cargo".into());
        if dep_lower.contains("wasm-bindgen") || dep_lower.contains("worker") {
            s.insert("backend".into(), "Rust WASM".into());
        } else if dep_lower.contains("axum") {
            s.insert("backend".into(), "Axum".into());
            s.insert("api_type".into(), "REST".into());
        } else if dep_lower.contains("actix") {
            s.insert("backend".into(), "Actix".into());
            s.insert("api_type".into(), "REST".into());
        } else if dep_lower.contains("rmcp") {
            s.insert("backend".into(), "Rust".into());
            s.insert("api_type".into(), "MCP".into());
        }
    }

    let _ = language;
    s
}

fn detect_version(dependency_file: Option<&str>) -> Option<String> {
    let dep = dependency_file?;
    // version = "x.y.z" (Cargo.toml / pyproject) or "version": "x.y.z" (package.json)
    if let Ok(re) = Regex::new(r#"(?m)^\s*version\s*[:=]\s*"?([0-9]+\.[0-9]+(?:\.[0-9]+)?)"?"#) {
        if let Some(c) = re.captures(dep) {
            return c.get(1).map(|m| m.as_str().to_string());
        }
    }
    if let Ok(re) = Regex::new(r#""version"\s*:\s*"([0-9]+\.[0-9]+(?:\.[0-9]+)?)""#) {
        if let Some(c) = re.captures(dep) {
            return c.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

fn extract_human_context(
    readme: Option<&str>,
    repo_name: &str,
    owner: &str,
    description: Option<&str>,
) -> HumanContext {
    let mut ctx = HumanContext::default();
    match readme {
        Some(r) => {
            ctx.who = extract_section(r, &["Authors?", "Contributors?", "Team"])
                .unwrap_or_else(|| format!("{} team", owner));
            ctx.what = extract_title(r)
                .or_else(|| description.map(|d| d.to_string()))
                .unwrap_or_else(|| repo_name.to_string());
            ctx.why = extract_section(r, &["Purpose", "Why", "Mission", "Goal"])
                .map(|t| first_sentences(&t, 3))
                .unwrap_or_default();
            ctx.where_ = extract_where(r);
            ctx.when = format!("Initialized {}", Utc::now().format("%Y-%m-%d"));
            ctx.how = extract_how(r);
        }
        None => {
            ctx.who = format!("{} team", owner);
            ctx.what = description.unwrap_or(repo_name).to_string();
            ctx.where_ = "GitHub".to_string();
            ctx.when = format!("Initialized {}", Utc::now().format("%Y-%m-%d"));
        }
    }
    ctx
}

fn extract_section(readme: &str, headings: &[&str]) -> Option<String> {
    for h in headings {
        let pat = format!(r"(?is)##\s*{}\s*\n(.*?)(?:\n##|\z)", h);
        if let Ok(re) = Regex::new(&pat) {
            if let Some(c) = re.captures(readme) {
                if let Some(m) = c.get(1) {
                    let t = m.as_str().trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_title(readme: &str) -> Option<String> {
    Regex::new(r"(?m)^#\s+(.+)$")
        .ok()
        .and_then(|re| re.captures(readme))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn extract_where(readme: &str) -> String {
    for d in &["Vercel", "AWS", "Heroku", "Netlify", "Cloudflare", "Docker", "Fly.io"] {
        if readme.contains(d) {
            return d.to_string();
        }
    }
    "GitHub".to_string()
}

fn extract_how(readme: &str) -> String {
    for (name, pat) in &[
        ("Installation", r"(?i)##\s*Installation"),
        ("Getting Started", r"(?i)##\s*Getting Started"),
        ("Usage", r"(?i)##\s*Usage"),
    ] {
        if Regex::new(pat).map(|re| re.is_match(readme)).unwrap_or(false) {
            return format!("See README: {}", name);
        }
    }
    String::new()
}

fn first_sentences(text: &str, max: usize) -> String {
    text.split('.').take(max).collect::<Vec<_>>().join(".").trim().to_string()
}

fn primary_stack(stack: &HashMap<String, String>, language: Option<&str>) -> String {
    stack
        .get("frontend")
        .or_else(|| stack.get("backend"))
        .or_else(|| stack.get("runtime"))
        .cloned()
        .or_else(|| language.map(|l| l.to_string()))
        .unwrap_or_default()
}

/// Render a YAML-safe scalar. Double-quotes any value that would otherwise be
/// mis-parsed — notably `: ` (colon-space reads as a nested mapping), leading
/// indicator chars, or trailing space. Bare colons (e.g. ISO timestamps) are
/// left unquoted to match faf-cli's canonical style.
fn yaml_scalar(v: &str) -> String {
    let needs_quote = v.contains(": ")
        || v.contains(" #")
        || v.contains('\n')
        || v.ends_with(':')
        || v.ends_with(' ')
        || v.starts_with(|c: char| "-?:,[]{}#&*!|>'\"%@` ".contains(c));
    if needs_quote {
        format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        v.to_string()
    }
}

/// `key: value` line (2-space indent). Empty value → bare `key:` (YAML null,
/// scores as empty), no trailing space.
fn kv(key: &str, value: &str) -> String {
    if value.is_empty() {
        format!("  {}:\n", key)
    } else {
        format!("  {}: {}\n", key, yaml_scalar(value))
    }
}

/// Emit a single slot line: detected value, empty (active+undetected), or `slotignored`.
fn slot_line(yaml: &mut String, key: &str, cat: Cat, active: &[Cat], stack: &HashMap<String, String>) {
    if active.contains(&cat) {
        match stack.get(key) {
            Some(v) => yaml.push_str(&kv(key, v)),
            None => yaml.push_str(&format!("  {}:\n", key)), // active but undetected → empty
        }
    } else {
        yaml.push_str(&format!("  {}: slotignored\n", key));
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_yaml(
    repo_name: &str,
    owner: &str,
    app_type: &str,
    stack: &HashMap<String, String>,
    human: &HumanContext,
    language: Option<&str>,
    primary: &str,
    version: Option<&str>,
    framework: Option<&str>,
) -> String {
    let active = active_cats(app_type);
    let lang = language.unwrap_or("");
    let mut y = String::new();

    // Header
    y.push_str("faf_version: \"3.3\"\n");

    // project
    y.push_str("project:\n");
    y.push_str(&kv("name", repo_name));
    y.push_str(&kv("goal", &human.what));
    y.push_str(&kv("main_language", lang));
    y.push_str(&kv("type", app_type));
    if let Some(v) = version {
        y.push_str(&kv("version", v));
    }
    if let Some(f) = framework {
        y.push_str(&kv("framework", f));
    }

    // instant_context
    y.push_str("instant_context:\n");
    y.push_str(&kv("what_building", &human.what));
    y.push_str(&kv("tech_stack", if primary.is_empty() { lang } else { primary }));
    y.push_str("  key_files: []\n");

    // stack (19 slots)
    y.push_str("stack:\n");
    for (key, cat) in STACK_SLOTS {
        slot_line(&mut y, key, *cat, &active, stack);
    }

    // human_context (6 slots)
    y.push_str("human_context:\n");
    y.push_str(&kv("who", &human.who));
    y.push_str(&kv("what", &human.what));
    y.push_str(&kv("why", &human.why));
    y.push_str(&kv("where", &human.where_));
    y.push_str(&kv("when", &human.when));
    y.push_str(&kv("how", &human.how));

    // tags
    y.push_str("tags:\n");
    y.push_str(&format!("  - {}\n", yaml_scalar(repo_name)));
    y.push_str(&format!("  - {}\n", yaml_scalar(app_type)));
    if !lang.is_empty() {
        y.push_str(&format!("  - {}\n", yaml_scalar(&lang.to_lowercase())));
    }
    y.push_str("  - faf\n");
    y.push_str("  - ai-context\n");

    // state
    y.push_str("state:\n");
    y.push_str("  phase: active\n");
    y.push_str(&kv("version", version.unwrap_or("1.0.0")));

    // metadata (generation attribution)
    y.push_str("metadata:\n");
    y.push_str("  generated_by: builder.faf.one\n");
    y.push_str(&kv("generated", &Utc::now().to_rfc3339()));
    y.push_str(&kv("repository", &format!("https://github.com/{}/{}", owner, repo_name)));

    // monorepo (5 slots)
    y.push_str("monorepo:\n");
    for (key, cat) in MONOREPO_SLOTS {
        // monorepo slots use the same key but live under monorepo:
        if active.contains(cat) {
            match stack.get(*key) {
                Some(v) => y.push_str(&format!("  {}: {}\n", key, v)),
                None => y.push_str(&format!("  {}:\n", key)),
            }
        } else {
            y.push_str(&format!("  {}: slotignored\n", key));
        }
    }

    y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_faf_version_3_3() {
        let y = generate_faf(
            "demo".into(), "owner".into(), None, None, None, Some("Rust".into()),
        )
        .unwrap();
        assert!(y.contains("faf_version: \"3.3\""), "must emit current faf_version");
    }

    #[test]
    fn no_obsolete_sections() {
        let y = generate_faf(
            "demo".into(), "owner".into(), None, None, None, Some("Rust".into()),
        )
        .unwrap();
        for dead in ["ai_tldr", "faf_dna", "ai_scoring_details", "ai_score", "context_quality"] {
            assert!(!y.contains(dead), "v6.8 dropped section `{}` must not appear", dead);
        }
    }

    #[test]
    fn mcp_type_detected_from_rmcp() {
        let cargo = "[package]\nname=\"x\"\nedition=\"2021\"\n[dependencies]\nrmcp=\"1.1\"";
        let t = detect_app_type(None, Some(cargo), Some("Rust"));
        assert_eq!(t, "mcp");
    }

    #[test]
    fn data_science_type_detected() {
        let py = "[tool.poetry]\nname=\"grok\"\n[tool.poetry.dependencies]\njax=\"^0.4\"";
        let t = detect_app_type(Some("# Grok\nmachine learning model"), Some(py), Some("Python"));
        assert_eq!(t, "data-science");
    }

    #[test]
    fn frontend_categories_slotignored_for_mcp() {
        // mcp = project+backend+human+universal → frontend slots are slotignored
        let cargo = "[package]\nname=\"x\"\nedition=\"2021\"\n[dependencies]\nrmcp=\"1.1\"";
        let y = generate_faf(
            "srv".into(), "me".into(), None, None, Some(cargo.into()), Some("Rust".into()),
        )
        .unwrap();
        assert!(y.contains("frontend: slotignored"), "frontend inactive for mcp");
        assert!(y.contains("css_framework: slotignored"));
        // backend IS active → backend slot must NOT be slotignored
        assert!(!y.contains("backend: slotignored"), "backend active for mcp");
    }

    #[test]
    fn all_33_slots_present() {
        let y = generate_faf(
            "x".into(), "o".into(), None, None, None, Some("Rust".into()),
        )
        .unwrap();
        // 19 stack + 5 monorepo slot keys must all appear
        for (k, _) in STACK_SLOTS.iter().chain(MONOREPO_SLOTS.iter()) {
            assert!(y.contains(&format!("{}:", k)), "missing slot `{}`", k);
        }
        // + project(3) + human(6)
        for k in ["name", "goal", "main_language", "who", "what", "why", "where", "when", "how"] {
            assert!(y.contains(&format!("{}:", k)), "missing field `{}`", k);
        }
    }

    #[test]
    fn enterprise_slots_slotignored_for_library() {
        let y = generate_faf(
            "lib".into(), "o".into(), None, None, None, Some("Rust".into()),
        )
        .unwrap();
        // library = project+human+universal → enterprise_app/ops slotignored
        assert!(y.contains("admin: slotignored"));
        assert!(y.contains("remote_cache: slotignored"));
    }
}
