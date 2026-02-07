# Rust WASM Generator Specification v1.0
**Target:** builder.faf.one DOUBLE-WHAMMY Architecture
**Bible:** faf-cli v4.2.1 (`/Users/wolfejam/FAF/cli`)
**Goal:** Generate project.faf in browser that scores identically to faf-cli

---

## Mission

Build a Rust WASM generator that:
1. **Matches faf-cli output exactly** for the same inputs
2. **Supports ALL project types** including ml-research (NEW)
3. **Omits undetected fields** (no "Unknown" hardcoding)
4. **Passes WJTTC test suite** (100% compatibility)

---

## Input Parameters

```rust
pub fn generate_faf(
    repo_name: String,
    owner: String,
    description: Option<String>,
    readme: Option<String>,
    dependency_file: Option<String>,  // package.json, pyproject.toml, Cargo.toml, go.mod, etc.
    language: Option<String>,          // From GitHub API
) -> Result<String, JsValue>
```

**Sources:**
- GitHub API (public): repo metadata, language, description
- GitHub API (public): README.md content
- GitHub API (public): dependency file (package.json, pyproject.toml, etc.)

**No assumptions.** Only fill what's detected.

---

## Project Type Detection

**Reference:** `/Users/wolfejam/FAF/cli/src/compiler/faf-compiler.ts` (TYPE_DEFINITIONS)

**Supported Types:**
```rust
enum ProjectType {
    WebApp,        // React, Vue, Svelte, Angular, Next.js
    BackendApi,    // Express, FastAPI, Django, Flask, Axum
    Cli,           // Commander, Clap, argparse
    Library,       // Reusable packages (default)
    MlModel,       // JAX, PyTorch, TensorFlow (ALIAS: ml-research)
    DataAnalysis,  // Jupyter, pandas, matplotlib
    FullStack,     // Next.js, SvelteKit, Nuxt
    Extension,     // Chrome/Firefox extensions
    Mobile,        // React Native, Flutter
    Desktop,       // Electron, Tauri
    Game,          // Unity, Godot, Bevy
}
```

**Detection Logic:**

1. **ML/AI** (highest priority):
   - pyproject.toml contains: jax, pytorch, tensorflow, flax, transformers
   - requirements.txt contains: torch, jax, tensorflow
   - README mentions: "machine learning", "deep learning", "neural network"
   - Type: `ml-research`

2. **Frontend frameworks**:
   - package.json dependencies: react, vue, svelte, @angular/core, next
   - Type: `web-app`

3. **Backend frameworks**:
   - package.json: express, fastapi, django, flask, axum, actix
   - Type: `backend-api`

4. **CLI tools**:
   - package.json: commander, yargs
   - Cargo.toml: clap
   - pyproject.toml: click, argparse, typer
   - Type: `cli`

5. **Default**: `library`

---

## Slot Categories (21 Total)

**Reference:** `/Users/wolfejam/FAF/cli/src/compiler/faf-compiler.ts` (ALL_SLOTS)

### Project Category (3 slots)
- `project.name` - Required
- `project.goal` - Required
- `project.main_language` - Required

### Frontend Category (3 slots)
- `stack.frontend`
- `stack.css_framework`
- `stack.ui_library`

### Backend Category (5 slots)
- `stack.backend`
- `stack.api_type`
- `stack.runtime`
- `stack.database`
- `stack.connection`

### Universal Category (4 slots)
- `stack.build`
- `stack.package_manager`
- `stack.hosting`
- `stack.cicd`

### Human Category (6 slots)
- `human_context.who`
- `human_context.what`
- `human_context.why`
- `human_context.where`
- `human_context.when`
- `human_context.how`

---

## Type-Specific Scoring

**ml-research** (14 slots):
- Categories: project (3) + backend (5) + human (6) = 14
- 100% = 14/14 filled

**web-app** (16 slots):
- Categories: project (3) + frontend (3) + universal (4) + human (6) = 16

**backend-api** (18 slots):
- Categories: project (3) + backend (5) + universal (4) + human (6) = 18

**library** (12 slots):
- Categories: project (3) + universal (3) + human (6) = 12

---

## Stack Detection Rules

### Python Projects (pyproject.toml)

**Detect from pyproject.toml:**
```rust
if dependency_file.contains("pyproject.toml") {
    // package_manager
    if content.contains("[tool.poetry]") {
        stack.package_manager = Some("poetry");
        stack.build = Some("poetry");
    }

    // runtime (always Python for pyproject.toml)
    stack.runtime = Some("Python");

    // backend (same as runtime for Python)
    stack.backend = Some("Python");

    // database (only if ORM detected)
    if content.contains("sqlalchemy") || content.contains("django.db") {
        stack.database = Some("SQL"); // Generic, let faf auto specify
    }

    // DO NOT assume database = "File-based" for ML projects
    // Leave empty if not detected
}
```

### JavaScript/TypeScript (package.json)

**Detect from package.json:**
```rust
if dependency_file.contains("package.json") {
    let pkg: PackageJson = parse_json(content)?;

    // Framework detection (priority order)
    if pkg.has_dependency("react") {
        stack.frontend = Some("React");
    } else if pkg.has_dependency("vue") {
        stack.frontend = Some("Vue");
    } else if pkg.has_dependency("svelte") {
        stack.frontend = Some("Svelte");
    }

    // Backend detection
    if pkg.has_dependency("express") {
        stack.backend = Some("Express");
        stack.api_type = Some("REST");
    } else if pkg.has_dependency("fastify") {
        stack.backend = Some("Fastify");
        stack.api_type = Some("REST");
    }

    // Runtime
    stack.runtime = Some("Node.js");

    // Package manager (default npm, detect others)
    stack.package_manager = Some("npm");

    // Build tool
    if pkg.has_dev_dependency("vite") {
        stack.build = Some("Vite");
    } else if pkg.has_dev_dependency("webpack") {
        stack.build = Some("Webpack");
    }
}
```

### Rust (Cargo.toml)

```rust
if dependency_file.contains("Cargo.toml") {
    stack.package_manager = Some("cargo");
    stack.runtime = Some("Native");
    stack.build = Some("cargo");

    if content.contains("wasm-bindgen") {
        stack.backend = Some("Rust WASM");
    } else if content.contains("axum") {
        stack.backend = Some("Axum");
        stack.api_type = Some("REST");
    } else if content.contains("actix") {
        stack.backend = Some("Actix");
        stack.api_type = Some("REST");
    }
}
```

---

## README Extraction (6 Ws)

**Reference:** `/Users/wolfejam/FAF/cli/src/commands/enhance-real.ts` (RelentlessContextExtractor)

### WHO
1. Extract from README "## Authors", "## Contributors", "## Team"
2. Fallback: `{owner} team`

### WHAT
1. Extract from README first `# Heading` (title)
2. Fallback: `repo_name`

### WHY
1. Search for sections: "## Purpose", "## Why", "## Mission", "## Goal"
2. Extract first 1-3 sentences
3. Fallback: `"TBD"`

### WHERE
1. Check for deployment mentions: "Vercel", "AWS", "Heroku", "Netlify"
2. Fallback: `"GitHub"`

### WHEN
1. Check for "## Roadmap", "## Timeline"
2. Fallback: `"Initialized {current_date}"`

### HOW
1. Check for "## Installation", "## Getting Started", "## Usage"
2. If found: `"See README: {section_name}"`
3. Fallback: `"TBD"`

---

## Field Omission Strategy

**CRITICAL:** Match faf-cli behavior exactly.

**Rule:** Only include fields with detected values. Omit the rest.

**Example:**
```yaml
# GOOD (faf-cli style)
stack:
  package_manager: poetry
  runtime: Python
  backend: Python

# BAD (old WASM style)
stack:
  primary: JAX
  frontend: JAX
  backend: Unknown      # ❌ Don't hardcode Unknown
  runtime: Node.js      # ❌ Wrong default
  database: Unknown     # ❌ Don't hardcode
```

**Implementation:**
```rust
// Build stack section dynamically
let mut stack_fields = Vec::new();

if let Some(pkg_mgr) = detected_package_manager {
    stack_fields.push(format!("  package_manager: {}", pkg_mgr));
}

if let Some(runtime) = detected_runtime {
    stack_fields.push(format!("  runtime: {}", runtime));
}

// Only join if we have fields
let stack_yaml = if stack_fields.is_empty() {
    "stack: null".to_string()
} else {
    format!("stack:\n{}", stack_fields.join("\n"))
};
```

---

## YAML Template Structure

**Match faf-cli v4.2.1 exactly:**

```yaml
faf_version: 2.5.0
ai_scoring_system: 2025-12-17
ai_confidence: MODERATE
ai_value: 30_seconds_replaces_20_minutes_of_questions

# AI-Optimized Context (The Quick Brief)
ai_tldr:
  project: {repo_name}
  stack: {primary_stack}
  quality_bar: PRODUCTION
  current_focus: Initial setup
  your_role: Build with AI assistance

# Instant Context Snapshot
instant_context:
  what_building: {what}
  tech_stack: {primary_stack}
  main_language: {language}
  deployment: {where}
  key_files: []

# Context Quality Metrics
context_quality:
  slots_filled: {filled}/{total} ({percentage}%)
  ai_confidence: MODERATE
  handoff_ready: {handoff_ready}
  missing_context:
    {missing_context_list}

# Project Identity
project:
  name: {repo_name}
  goal: {goal}
  main_language: {language}
  type: {project_type}
  version: {version}
  generated: {timestamp}
  repository: https://github.com/{owner}/{repo_name}

# AI Instructions
ai_instructions:
  priority_order:
    - 1. Read THIS .faf file first
    - 2. Check README.md for overview
    - 3. Review key files
  working_style:
    code_first: true
    explanations: clear
    quality_bar: production
    testing: recommended
  warnings:
    - Follow existing code patterns
    - Test before committing

# Technology Stack
stack:
  {dynamically_generated_stack_fields}

# Developer Preferences
preferences:
  quality_bar: production
  commit_style: conventional
  response_style: balanced
  explanation_level: clear
  communication: friendly
  testing: recommended

# Project State
state:
  phase: active
  version: {version}
  focus: development
  status: ready
  next_milestone: Define roadmap
  blockers: null

# Tags
tags:
  - {repo_name}
  - {primary_stack}
  - faf
  - ai-ready

# Human Context (The 6 Ws)
human_context:
  who: {who}
  what: {what}
  why: {why}
  where: {where}
  when: {when}
  how: {how}
  additional_context: Generated by builder.faf.one
  context_score: {percentage}
  total_prd_score: {percentage}
  success_rate: {percentage}%

# AI Scoring Details
ai_scoring_details:
  system_date: 2025-12-17
  slot_based_percentage: {percentage}
  ai_score: {percentage}
  total_slots: {total}
  filled_slots: {filled}
  scoring_method: Honest percentage - no fake minimums
  trust_embedded: COUNT ONCE architecture

# FAF DNA (Birth Certificate)
faf_dna:
  birth_dna: {percentage}
  birth_certificate: FAF-2025-{repo_upper}-INIT
  birth_date: {timestamp}
  current_score: {percentage}
```

---

## Test Suite (WJTTC)

**Must pass ALL tests against faf-cli as ground truth:**

### Test Cases

1. **test-faf-demo** (JavaScript/React)
   - Expected type: `web-app`
   - Expected stack: React, npm, Node.js
   - Score: Should match faf-cli

2. **grok-1** (Python/JAX)
   - Expected type: `ml-research`
   - Expected stack: poetry, Python, JAX
   - Score: Should match faf-cli

3. **faf-cli** (TypeScript)
   - Expected type: `cli`
   - Expected stack: npm, Node.js, TypeScript
   - Score: Should match faf-cli

4. **Empty repo** (no files)
   - Expected: Minimal .faf with repo metadata only
   - Score: Low (5-10%)

5. **Private repo** (metadata only)
   - Expected: Basic .faf from GitHub API metadata
   - Score: Low (20-30%)

**Test Harness:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_faf_cli_output() {
        let wasm_output = generate_faf(...);
        let faf_cli_output = read_expected_output("test-cases/grok-1.faf");

        // Parse both as YAML
        let wasm_parsed: FAF = parse_yaml(&wasm_output);
        let cli_parsed: FAF = parse_yaml(&faf_cli_output);

        // Compare slot by slot
        assert_eq!(wasm_parsed.project.name, cli_parsed.project.name);
        assert_eq!(wasm_parsed.project.type, cli_parsed.project.type);
        // ... all fields
    }

    #[test]
    fn test_ml_research_type_detection() {
        let result = generate_faf(
            "grok-1",
            "xai-org",
            Some("Grok open release"),
            Some(include_str!("fixtures/grok-1-readme.md")),
            Some(include_str!("fixtures/pyproject.toml")),
            Some("Python")
        );

        assert!(result.contains("type: ml-research"));
        assert!(result.contains("runtime: Python"));
        assert!(result.contains("package_manager: poetry"));
    }
}
```

---

## Build & Distribution

**Build:**
```bash
cd /Users/wolfejam/FAF/faf-wasm-sdk
wasm-pack build --target web --release
```

**Output:**
- `pkg/faf_wasm_sdk_bg.wasm` (WASM binary)
- `pkg/faf_wasm_sdk.js` (JS glue code)

**Target size:** ~200-250KB (acceptable for web)

**Publish:**
```bash
cd pkg
npm publish
```

**Integration:**
```javascript
import init, { generate_faf } from '@faf/wasm-sdk';

await init(); // Load WASM

const faf = generate_faf(
    'grok-1',
    'xai-org',
    'Grok open release',
    readmeContent,
    pyprojectContent,
    'Python'
);

console.log(faf); // Valid project.faf YAML
```

---

## Success Criteria

**WASM generator passes if:**
1. ✅ Generates identical .faf to faf-cli for same inputs (slot-by-slot match)
2. ✅ Supports ml-research type (NEW - critical for Grok-1 demo)
3. ✅ Omits undetected fields (no "Unknown" hardcoding)
4. ✅ Scores identically to faf-cli when run through scorer
5. ✅ Passes WJTTC test suite (100% compatibility)
6. ✅ Works with builder.faf.one + Zig scorer (DOUBLE-WHAMMY)

---

## Non-Goals (Out of Scope)

- ❌ Turbo-Cat format discovery (that's faf-cli only)
- ❌ AI enhancement (that's faf-cli `--ai` flag)
- ❌ bi-sync with CLAUDE.md (that's faf-cli)
- ❌ FAFb binary compilation (that's xai-faf-rust)

**This is a generator ONLY.** It creates the initial project.faf.

---

## Reference Implementation

**Bible:** `/Users/wolfejam/FAF/cli/src/generators/faf-generator-championship.ts`

**Key files to study:**
- Type detection: `/Users/wolfejam/FAF/cli/src/compiler/faf-compiler.ts` (TYPE_DEFINITIONS)
- Context extraction: `/Users/wolfejam/FAF/cli/src/commands/enhance-real.ts` (RelentlessContextExtractor)
- Stack detection: `/Users/wolfejam/FAF/cli/src/utils/file-utils.ts` (detectProjectType)

---

## Deliverables for Opus 4.6

1. **Complete Rust WASM generator** matching this spec
2. **WJTTC test suite** (5+ test cases, all passing)
3. **Documentation** (README with usage examples)
4. **Build verified** (wasm-pack builds successfully)
5. **npm ready** (package.json, publishable)

---

**Status:** Ready for implementation
**Owner:** Opus 4.6
**Timeline:** TBD
**Champion:** wolfejam 🏎️⚡️

---

*Spec version: 1.0*
*Date: 2026-02-07*
*FAF Ecosystem: Championship Edition*
