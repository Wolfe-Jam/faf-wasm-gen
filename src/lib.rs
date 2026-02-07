/*!
 * faf-generator-wasm
 *
 * Rust WASM Generator for FAF (Foundational AI-context Format)
 *
 * SPEC: See ../SPEC.md for complete implementation requirements
 * BIBLE: /Users/wolfejam/FAF/cli (faf-cli v4.2.1)
 *
 * MISSION: Generate project.faf that matches faf-cli output exactly
 */

use wasm_bindgen::prelude::*;
use serde_json::Value;
use chrono::Utc;
use regex::Regex;

#[derive(Debug, Clone)]
enum ProjectType {
    WebApp,
    BackendApi,
    Cli,
    Library,
    MlModel,      // Aliases: ml-research
    DataAnalysis,
    FullStack,
    Extension,
    Mobile,
    Desktop,
    Game,
}

impl ProjectType {
    fn as_str(&self) -> &str {
        match self {
            ProjectType::WebApp => "web-app",
            ProjectType::BackendApi => "backend-api",
            ProjectType::Cli => "cli",
            ProjectType::Library => "library",
            ProjectType::MlModel => "ml-research",
            ProjectType::DataAnalysis => "data-analysis",
            ProjectType::FullStack => "full-stack",
            ProjectType::Extension => "extension",
            ProjectType::Mobile => "mobile",
            ProjectType::Desktop => "desktop",
            ProjectType::Game => "game",
        }
    }

    fn slot_count(&self) -> usize {
        match self {
            ProjectType::MlModel => 14,       // project (3) + backend (5) + human (6)
            ProjectType::WebApp => 16,        // project (3) + frontend (3) + universal (4) + human (6)
            ProjectType::BackendApi => 18,    // project (3) + backend (5) + universal (4) + human (6)
            ProjectType::Library => 12,       // project (3) + universal (3) + human (6)
            _ => 12,                          // Default to library slots
        }
    }
}

#[derive(Debug, Default)]
struct Stack {
    frontend: Option<String>,
    css_framework: Option<String>,
    ui_library: Option<String>,
    backend: Option<String>,
    api_type: Option<String>,
    runtime: Option<String>,
    database: Option<String>,
    connection: Option<String>,
    build: Option<String>,
    package_manager: Option<String>,
    hosting: Option<String>,
    cicd: Option<String>,
}

#[derive(Debug, Default)]
struct HumanContext {
    who: String,
    what: String,
    why: String,
    where_: String,
    when: String,
    how: String,
}

/// Generate project.faf from repository metadata
///
/// # Parameters
/// - `repo_name`: Repository name (e.g., "grok-1")
/// - `owner`: Repository owner (e.g., "xai-org")
/// - `description`: Optional repository description
/// - `readme`: Optional README.md content
/// - `dependency_file`: Optional dependency file (package.json, pyproject.toml, etc.)
/// - `language`: Optional primary language from GitHub API
///
/// # Returns
/// Generated project.faf YAML content
///
/// # Example
/// ```javascript
/// import init, { generate_faf } from './faf_generator_wasm.js';
///
/// await init();
///
/// const faf = generate_faf(
///     'grok-1',
///     'xai-org',
///     'Grok open release',
///     readmeContent,
///     pyprojectContent,
///     'Python'
/// );
/// ```
#[wasm_bindgen]
pub fn generate_faf(
    repo_name: String,
    owner: String,
    description: Option<String>,
    readme: Option<String>,
    dependency_file: Option<String>,
    language: Option<String>,
) -> Result<String, JsValue> {
    // Detect project type
    let project_type = detect_project_type(
        readme.as_deref(),
        dependency_file.as_deref(),
        language.as_deref(),
    );

    // Extract stack information
    let stack = detect_stack(dependency_file.as_deref(), language.as_deref());

    // Extract human context (6 Ws)
    let human_context = extract_human_context(
        readme.as_deref(),
        &repo_name,
        &owner,
        description.as_deref(),
    );

    // Calculate filled slots
    let (filled_slots, total_slots) = calculate_filled_slots(&project_type, &stack, &human_context);
    let percentage = (filled_slots * 100) / total_slots;

    // Determine primary stack
    let primary_stack = determine_primary_stack(&stack, language.as_deref());

    // Generate YAML
    let yaml = generate_yaml(
        &repo_name,
        &owner,
        &project_type,
        &stack,
        &human_context,
        language.as_deref(),
        &primary_stack,
        filled_slots,
        total_slots,
        percentage,
    );

    Ok(yaml)
}

fn detect_project_type(
    readme: Option<&str>,
    dependency_file: Option<&str>,
    language: Option<&str>,
) -> ProjectType {
    let readme_lower = readme.map(|s| s.to_lowercase()).unwrap_or_default();
    let dep_file_lower = dependency_file.map(|s| s.to_lowercase()).unwrap_or_default();

    // Priority 1: ML/AI detection (highest priority)
    if dep_file_lower.contains("jax") ||
       dep_file_lower.contains("pytorch") ||
       dep_file_lower.contains("tensorflow") ||
       dep_file_lower.contains("torch") ||
       dep_file_lower.contains("flax") ||
       dep_file_lower.contains("transformers") ||
       readme_lower.contains("machine learning") ||
       readme_lower.contains("deep learning") ||
       readme_lower.contains("neural network") {
        return ProjectType::MlModel;
    }

    // Priority 2: Frontend frameworks
    if dep_file_lower.contains("\"react\"") ||
       dep_file_lower.contains("\"vue\"") ||
       dep_file_lower.contains("\"svelte\"") ||
       dep_file_lower.contains("@angular/core") ||
       dep_file_lower.contains("\"next\"") {
        return ProjectType::WebApp;
    }

    // Priority 3: Backend frameworks
    if dep_file_lower.contains("\"express\"") ||
       dep_file_lower.contains("\"fastapi\"") ||
       dep_file_lower.contains("\"django\"") ||
       dep_file_lower.contains("\"flask\"") ||
       dep_file_lower.contains("\"axum\"") ||
       dep_file_lower.contains("\"actix\"") {
        return ProjectType::BackendApi;
    }

    // Priority 4: CLI tools
    if dep_file_lower.contains("\"commander\"") ||
       dep_file_lower.contains("\"yargs\"") ||
       dep_file_lower.contains("\"clap\"") ||
       dep_file_lower.contains("\"click\"") ||
       dep_file_lower.contains("\"argparse\"") ||
       dep_file_lower.contains("\"typer\"") {
        return ProjectType::Cli;
    }

    // Default: library
    ProjectType::Library
}

fn detect_stack(dependency_file: Option<&str>, language: Option<&str>) -> Stack {
    let mut stack = Stack::default();

    if let Some(dep_content) = dependency_file {
        let dep_lower = dep_content.to_lowercase();

        // Detect Python stack (pyproject.toml)
        if dep_content.contains("[tool.poetry]") || dep_content.contains("pyproject.toml") {
            if dep_content.contains("[tool.poetry]") {
                stack.package_manager = Some("poetry".to_string());
                stack.build = Some("poetry".to_string());
            }
            stack.runtime = Some("Python".to_string());
            stack.backend = Some("Python".to_string());

            // Database detection (only if ORM detected)
            if dep_lower.contains("sqlalchemy") || dep_lower.contains("django.db") {
                stack.database = Some("SQL".to_string());
            }
        }

        // Detect JavaScript/TypeScript stack (package.json)
        if let Ok(pkg_json) = serde_json::from_str::<Value>(dep_content) {
            if let Some(deps) = pkg_json.get("dependencies").and_then(|d| d.as_object()) {
                // Frontend detection
                if deps.contains_key("react") {
                    stack.frontend = Some("React".to_string());
                } else if deps.contains_key("vue") {
                    stack.frontend = Some("Vue".to_string());
                } else if deps.contains_key("svelte") {
                    stack.frontend = Some("Svelte".to_string());
                } else if deps.contains_key("@angular/core") {
                    stack.frontend = Some("Angular".to_string());
                }

                // Backend detection
                if deps.contains_key("express") {
                    stack.backend = Some("Express".to_string());
                    stack.api_type = Some("REST".to_string());
                } else if deps.contains_key("fastify") {
                    stack.backend = Some("Fastify".to_string());
                    stack.api_type = Some("REST".to_string());
                }

                // Runtime
                stack.runtime = Some("Node.js".to_string());
                stack.package_manager = Some("npm".to_string());
            }

            // Build tool detection
            if let Some(dev_deps) = pkg_json.get("devDependencies").and_then(|d| d.as_object()) {
                if dev_deps.contains_key("vite") {
                    stack.build = Some("Vite".to_string());
                } else if dev_deps.contains_key("webpack") {
                    stack.build = Some("Webpack".to_string());
                }
            }
        }

        // Detect Rust stack (Cargo.toml)
        if dep_content.contains("[package]") && dep_content.contains("cargo") {
            stack.package_manager = Some("cargo".to_string());
            stack.runtime = Some("Native".to_string());
            stack.build = Some("cargo".to_string());

            if dep_lower.contains("wasm-bindgen") {
                stack.backend = Some("Rust WASM".to_string());
            } else if dep_lower.contains("axum") {
                stack.backend = Some("Axum".to_string());
                stack.api_type = Some("REST".to_string());
            } else if dep_lower.contains("actix") {
                stack.backend = Some("Actix".to_string());
                stack.api_type = Some("REST".to_string());
            }
        }
    }

    stack
}

fn extract_human_context(
    readme: Option<&str>,
    repo_name: &str,
    owner: &str,
    description: Option<&str>,
) -> HumanContext {
    let mut context = HumanContext::default();

    if let Some(readme_content) = readme {
        // Extract WHO
        context.who = extract_who(readme_content, owner);

        // Extract WHAT
        context.what = extract_what(readme_content, repo_name, description);

        // Extract WHY
        context.why = extract_why(readme_content);

        // Extract WHERE
        context.where_ = extract_where(readme_content);

        // Extract WHEN
        context.when = extract_when();

        // Extract HOW
        context.how = extract_how(readme_content);
    } else {
        // Fallback values when no README
        context.who = format!("{} team", owner);
        context.what = description.unwrap_or(repo_name).to_string();
        context.why = "TBD".to_string();
        context.where_ = "GitHub".to_string();
        context.when = format!("Initialized {}", Utc::now().format("%Y-%m-%d"));
        context.how = "TBD".to_string();
    }

    context
}

fn extract_who(readme: &str, owner: &str) -> String {
    // Look for sections like "## Authors", "## Contributors", "## Team"
    let patterns = vec![
        r"##\s*Authors?\s*\n(.*?)\n",
        r"##\s*Contributors?\s*\n(.*?)\n",
        r"##\s*Team\s*\n(.*?)\n",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(readme) {
                if let Some(match_text) = cap.get(1) {
                    let text = match_text.as_str().trim();
                    if !text.is_empty() {
                        return text.to_string();
                    }
                }
            }
        }
    }

    format!("{} team", owner)
}

fn extract_what(readme: &str, repo_name: &str, description: Option<&str>) -> String {
    // Extract first # heading (title)
    if let Ok(re) = Regex::new(r"#\s+(.+)") {
        if let Some(cap) = re.captures(readme) {
            if let Some(match_text) = cap.get(1) {
                return match_text.as_str().trim().to_string();
            }
        }
    }

    // Fallback to description or repo name
    description.unwrap_or(repo_name).to_string()
}

fn extract_why(readme: &str) -> String {
    // Look for sections like "## Purpose", "## Why", "## Mission", "## Goal"
    let patterns = vec![
        r"##\s*Purpose\s*\n(.*?)(?:\n##|\z)",
        r"##\s*Why\s*\n(.*?)(?:\n##|\z)",
        r"##\s*Mission\s*\n(.*?)(?:\n##|\z)",
        r"##\s*Goal\s*\n(.*?)(?:\n##|\z)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(readme) {
                if let Some(match_text) = cap.get(1) {
                    let text = match_text.as_str().trim();
                    if !text.is_empty() {
                        // Extract first 1-3 sentences
                        return extract_first_sentences(text, 3);
                    }
                }
            }
        }
    }

    "TBD".to_string()
}

fn extract_where(readme: &str) -> String {
    // Check for deployment mentions
    let deployments = vec!["Vercel", "AWS", "Heroku", "Netlify", "Cloudflare"];

    for deployment in deployments {
        if readme.contains(deployment) {
            return deployment.to_string();
        }
    }

    "GitHub".to_string()
}

fn extract_when() -> String {
    format!("Initialized {}", Utc::now().format("%Y-%m-%d"))
}

fn extract_how(readme: &str) -> String {
    // Check for sections like "## Installation", "## Getting Started", "## Usage"
    let sections = vec![
        ("Installation", r"##\s*Installation"),
        ("Getting Started", r"##\s*Getting Started"),
        ("Usage", r"##\s*Usage"),
    ];

    for (name, pattern) in sections {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(readme) {
                return format!("See README: {}", name);
            }
        }
    }

    "TBD".to_string()
}

fn extract_first_sentences(text: &str, max_count: usize) -> String {
    let sentences: Vec<&str> = text
        .split('.')
        .take(max_count)
        .collect();

    sentences.join(".").trim().to_string()
}

fn calculate_filled_slots(project_type: &ProjectType, stack: &Stack, human_context: &HumanContext) -> (usize, usize) {
    let total_slots = project_type.slot_count();
    let mut filled = 3; // Project slots always filled (name, goal, main_language)

    // Count stack slots
    if stack.frontend.is_some() { filled += 1; }
    if stack.css_framework.is_some() { filled += 1; }
    if stack.ui_library.is_some() { filled += 1; }
    if stack.backend.is_some() { filled += 1; }
    if stack.api_type.is_some() { filled += 1; }
    if stack.runtime.is_some() { filled += 1; }
    if stack.database.is_some() { filled += 1; }
    if stack.connection.is_some() { filled += 1; }
    if stack.build.is_some() { filled += 1; }
    if stack.package_manager.is_some() { filled += 1; }
    if stack.hosting.is_some() { filled += 1; }
    if stack.cicd.is_some() { filled += 1; }

    // Count human context slots (6 Ws)
    if !human_context.who.is_empty() && human_context.who != "TBD" { filled += 1; }
    if !human_context.what.is_empty() && human_context.what != "TBD" { filled += 1; }
    if !human_context.why.is_empty() && human_context.why != "TBD" { filled += 1; }
    if !human_context.where_.is_empty() && human_context.where_ != "TBD" { filled += 1; }
    if !human_context.when.is_empty() && human_context.when != "TBD" { filled += 1; }
    if !human_context.how.is_empty() && human_context.how != "TBD" { filled += 1; }

    (filled, total_slots)
}

fn determine_primary_stack(stack: &Stack, language: Option<&str>) -> String {
    if let Some(frontend) = &stack.frontend {
        return frontend.clone();
    }
    if let Some(backend) = &stack.backend {
        return backend.clone();
    }
    if let Some(runtime) = &stack.runtime {
        return runtime.clone();
    }
    language.unwrap_or("Unknown").to_string()
}

fn generate_yaml(
    repo_name: &str,
    owner: &str,
    project_type: &ProjectType,
    stack: &Stack,
    human_context: &HumanContext,
    language: Option<&str>,
    primary_stack: &str,
    filled_slots: usize,
    total_slots: usize,
    percentage: usize,
) -> String {
    let timestamp = Utc::now().to_rfc3339();
    let repo_upper = repo_name.to_uppercase().replace("-", "");
    let birth_cert = format!("FAF-2025-{}-INIT", &repo_upper[..std::cmp::min(8, repo_upper.len())]);

    let mut yaml = String::new();

    // Header
    yaml.push_str("faf_version: 2.5.0\n");
    yaml.push_str(&format!("generated: {}\n", timestamp));
    yaml.push_str("ai_scoring_system: 2025-12-17\n");
    yaml.push_str(&format!("ai_score: {}%\n", percentage));
    yaml.push_str("ai_confidence: MODERATE\n");
    yaml.push_str("ai_value: 30_seconds_replaces_20_minutes_of_questions\n\n");

    // AI TL;DR
    yaml.push_str("ai_tldr:\n");
    yaml.push_str(&format!("  project: {}\n", repo_name));
    yaml.push_str(&format!("  stack: {}\n", primary_stack));
    yaml.push_str("  quality_bar: PRODUCTION\n");
    yaml.push_str("  current_focus: Initial setup\n");
    yaml.push_str("  your_role: Build with AI assistance\n\n");

    // Instant Context
    yaml.push_str("instant_context:\n");
    yaml.push_str(&format!("  what_building: {}\n", human_context.what));
    yaml.push_str(&format!("  tech_stack: {}\n", primary_stack));
    yaml.push_str(&format!("  main_language: {}\n", language.unwrap_or("Unknown")));
    yaml.push_str(&format!("  deployment: {}\n", human_context.where_));
    yaml.push_str("  key_files: []\n\n");

    // Context Quality
    yaml.push_str("context_quality:\n");
    yaml.push_str(&format!("  slots_filled: {}/{} ({}%)\n", filled_slots, total_slots, percentage));
    yaml.push_str("  ai_confidence: MODERATE\n");
    yaml.push_str(&format!("  handoff_ready: {}\n", if percentage >= 85 { "true" } else { "false" }));
    yaml.push_str("  missing_context:\n");
    // TODO: List missing fields
    yaml.push_str("    - Additional context needed\n\n");

    // Project
    yaml.push_str("project:\n");
    yaml.push_str(&format!("  name: {}\n", repo_name));
    yaml.push_str(&format!("  goal: {}\n", human_context.what));
    yaml.push_str(&format!("  main_language: {}\n", language.unwrap_or("Unknown")));
    yaml.push_str(&format!("  type: {}\n", project_type.as_str()));
    yaml.push_str("  version: 1.0.0\n");
    yaml.push_str(&format!("  generated: {}\n", timestamp));
    yaml.push_str(&format!("  repository: https://github.com/{}/{}\n\n", owner, repo_name));

    // AI Instructions
    yaml.push_str("ai_instructions:\n");
    yaml.push_str("  priority_order:\n");
    yaml.push_str("    - 1. Read THIS .faf file first\n");
    yaml.push_str("    - 2. Check README.md for overview\n");
    yaml.push_str("    - 3. Review key files\n");
    yaml.push_str("  working_style:\n");
    yaml.push_str("    code_first: true\n");
    yaml.push_str("    explanations: clear\n");
    yaml.push_str("    quality_bar: production\n");
    yaml.push_str("    testing: recommended\n");
    yaml.push_str("  warnings:\n");
    yaml.push_str("    - Follow existing code patterns\n");
    yaml.push_str("    - Test before committing\n\n");

    // Stack (only include detected fields)
    yaml.push_str("stack:\n");
    if let Some(frontend) = &stack.frontend {
        yaml.push_str(&format!("  frontend: {}\n", frontend));
    }
    if let Some(css) = &stack.css_framework {
        yaml.push_str(&format!("  css_framework: {}\n", css));
    }
    if let Some(ui) = &stack.ui_library {
        yaml.push_str(&format!("  ui_library: {}\n", ui));
    }
    if let Some(backend) = &stack.backend {
        yaml.push_str(&format!("  backend: {}\n", backend));
    }
    if let Some(api) = &stack.api_type {
        yaml.push_str(&format!("  api_type: {}\n", api));
    }
    if let Some(runtime) = &stack.runtime {
        yaml.push_str(&format!("  runtime: {}\n", runtime));
    }
    if let Some(db) = &stack.database {
        yaml.push_str(&format!("  database: {}\n", db));
    }
    if let Some(conn) = &stack.connection {
        yaml.push_str(&format!("  connection: {}\n", conn));
    }
    if let Some(build) = &stack.build {
        yaml.push_str(&format!("  build: {}\n", build));
    }
    if let Some(pkg_mgr) = &stack.package_manager {
        yaml.push_str(&format!("  package_manager: {}\n", pkg_mgr));
    }
    if let Some(hosting) = &stack.hosting {
        yaml.push_str(&format!("  hosting: {}\n", hosting));
    }
    if let Some(cicd) = &stack.cicd {
        yaml.push_str(&format!("  cicd: {}\n", cicd));
    }
    yaml.push_str("\n");

    // Preferences
    yaml.push_str("preferences:\n");
    yaml.push_str("  quality_bar: production\n");
    yaml.push_str("  commit_style: conventional\n");
    yaml.push_str("  response_style: balanced\n");
    yaml.push_str("  explanation_level: clear\n");
    yaml.push_str("  communication: friendly\n");
    yaml.push_str("  testing: recommended\n\n");

    // State
    yaml.push_str("state:\n");
    yaml.push_str("  phase: active\n");
    yaml.push_str("  version: 1.0.0\n");
    yaml.push_str("  focus: development\n");
    yaml.push_str("  status: ready\n");
    yaml.push_str("  next_milestone: Define roadmap\n");
    yaml.push_str("  blockers: null\n\n");

    // Tags
    yaml.push_str("tags:\n");
    yaml.push_str(&format!("  - {}\n", repo_name));
    yaml.push_str(&format!("  - {}\n", primary_stack));
    yaml.push_str("  - faf\n");
    yaml.push_str("  - ai-ready\n\n");

    // Human Context
    yaml.push_str("human_context:\n");
    yaml.push_str(&format!("  who: {}\n", human_context.who));
    yaml.push_str(&format!("  what: {}\n", human_context.what));
    yaml.push_str(&format!("  why: {}\n", human_context.why));
    yaml.push_str(&format!("  where: {}\n", human_context.where_));
    yaml.push_str(&format!("  when: {}\n", human_context.when));
    yaml.push_str(&format!("  how: {}\n", human_context.how));
    yaml.push_str("  additional_context: Generated by builder.faf.one\n");
    yaml.push_str(&format!("  context_score: {}\n", percentage));
    yaml.push_str(&format!("  total_prd_score: {}\n", percentage));
    yaml.push_str(&format!("  success_rate: {}%\n\n", percentage));

    // AI Scoring Details
    yaml.push_str("ai_scoring_details:\n");
    yaml.push_str("  system_date: 2025-12-17\n");
    yaml.push_str(&format!("  slot_based_percentage: {}\n", percentage));
    yaml.push_str(&format!("  ai_score: {}\n", percentage));
    yaml.push_str(&format!("  total_slots: {}\n", total_slots));
    yaml.push_str(&format!("  filled_slots: {}\n", filled_slots));
    yaml.push_str("  scoring_method: Honest percentage - no fake minimums\n");
    yaml.push_str("  trust_embedded: COUNT ONCE architecture\n\n");

    // FAF DNA
    yaml.push_str("faf_dna:\n");
    yaml.push_str(&format!("  birth_dna: {}\n", percentage));
    yaml.push_str(&format!("  birth_certificate: {}\n", birth_cert));
    yaml.push_str(&format!("  birth_date: {}\n", timestamp));
    yaml.push_str(&format!("  current_score: {}\n", percentage));

    yaml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_research_type_detection() {
        let pyproject = r#"
[tool.poetry]
name = "grok-1"

[tool.poetry.dependencies]
jax = "^0.4.0"
flax = "^0.7.0"
"#;

        let readme = "# Grok-1\n\nA machine learning model.";

        let project_type = detect_project_type(Some(readme), Some(pyproject), Some("Python"));
        assert_eq!(project_type.as_str(), "ml-research");
    }

    #[test]
    fn test_web_app_detection() {
        let package_json = r#"{
  "dependencies": {
    "react": "^18.0.0"
  }
}"#;

        let project_type = detect_project_type(None, Some(package_json), Some("JavaScript"));
        assert_eq!(project_type.as_str(), "web-app");
    }

    #[test]
    fn test_stack_detection_python() {
        let pyproject = r#"
[tool.poetry]
name = "test"

[tool.poetry.dependencies]
python = "^3.8"
"#;

        let stack = detect_stack(Some(pyproject), Some("Python"));
        assert_eq!(stack.package_manager, Some("poetry".to_string()));
        assert_eq!(stack.runtime, Some("Python".to_string()));
    }

    #[test]
    fn test_no_unknown_hardcoding() {
        let result = generate_faf(
            "test-repo".to_string(),
            "test-owner".to_string(),
            None,
            None,
            None,
            Some("Rust".to_string()),
        );

        assert!(result.is_ok());
        let yaml = result.unwrap();
        assert!(!yaml.contains("Unknown"), "Should not contain 'Unknown' hardcoding");
    }
}
