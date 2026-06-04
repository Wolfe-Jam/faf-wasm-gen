/*!
 * Integration tests for faf-wasm-gen (faf-cli v6.8 / faf_version "3.3")
 *
 * Assert the CURRENT Mk4 model:
 *   - canonical app types (data-science, frontend, library, mcp …)
 *   - 33-slot output; slots outside the type's active categories = `slotignored`
 *     (a DETECTED value in an inactive category is still slotignored — the type
 *     decides which categories are scored)
 *   - score is NOT stored in the file (faf-cli computes populated/active on read)
 */

use faf_wasm_gen::generate_faf;

#[test]
fn test_grok1_data_science_detection() {
    let readme = r#"
# Grok-1

This repository contains JAX example code for Grok-1 open-weights model.

## Model Architecture

Grok-1 is a 314B parameter Mixture-of-Experts model trained by xAI.

## Installation

```bash
pip install -r requirements.txt
```
"#;

    let pyproject = r#"
[tool.poetry]
name = "grok-1"
version = "1.0.0"
description = "Grok-1 open release"

[tool.poetry.dependencies]
python = "^3.10"
jax = { version = "0.4.20", extras = ["cuda12_pip"] }
flax = "^0.7.5"
"#;

    let faf = generate_faf(
        "grok-1".to_string(),
        "xai-org".to_string(),
        Some("Grok-1 open release".to_string()),
        Some(readme.to_string()),
        Some(pyproject.to_string()),
        Some("Python".to_string()),
    )
    .expect("generation should succeed");

    assert!(faf.contains("faf_version: \"3.3\""), "current faf_version");
    // ML deps → canonical data-science type (was the old "ml-research")
    assert!(faf.contains("type: data-science"), "data-science type");
    assert!(faf.contains("name: grok-1"));
    // data-science = project+backend+human+universal → backend+universal active
    assert!(faf.contains("runtime: Python"), "runtime active (backend)");
    assert!(faf.contains("build: poetry"), "build active (universal)");
    // frontend category is NOT active for data-science → slotignored
    assert!(faf.contains("frontend: slotignored"));
    // package_manager is enterprise_infra → slotignored even though poetry detected
    assert!(faf.contains("package_manager: slotignored"));
    // 6 Ws
    assert!(faf.contains("who: xai-org team"));
    assert!(faf.contains("what: Grok-1"));
    assert!(faf.contains("where: GitHub"));
    // value contains ": " so it's YAML-quoted
    assert!(faf.contains("how: \"See README: Installation\""));
}

#[test]
fn test_react_frontend_detection() {
    let readme = r#"
# My React App

A modern React application.

## Getting Started

Install dependencies and start the dev server.
"#;

    let package_json = r#"{
  "name": "my-react-app",
  "version": "2.1.0",
  "dependencies": { "react": "^18.2.0", "react-dom": "^18.2.0" },
  "devDependencies": { "vite": "^5.0.0" }
}"#;

    let faf = generate_faf(
        "my-react-app".to_string(),
        "test-org".to_string(),
        Some("A modern React application".to_string()),
        Some(readme.to_string()),
        Some(package_json.to_string()),
        Some("JavaScript".to_string()),
    )
    .expect("generation should succeed");

    // Canonical frontend type (was old "web-app")
    assert!(faf.contains("type: frontend"), "frontend type");
    assert!(faf.contains("frontend: React"), "frontend slot active+detected");
    assert!(faf.contains("build: Vite"), "build active (universal)");
    // backend (runtime) inactive for frontend → slotignored even though Node.js detected
    assert!(faf.contains("runtime: slotignored"));
    // version flows through from package.json
    assert!(faf.contains("version: 2.1.0"));
}

#[test]
fn test_minimal_rust_library_defaults() {
    let faf = generate_faf(
        "minimal-repo".to_string(),
        "test-user".to_string(),
        None,
        None,
        None,
        Some("Rust".to_string()),
    )
    .expect("generation should succeed");

    assert!(faf.contains("faf_version: \"3.3\""));
    assert!(faf.contains("name: minimal-repo"));
    assert!(faf.contains("type: library"), "default to library");
    assert!(!faf.contains("Unknown"), "no Unknown hardcoding");
    // library = project+human+universal → backend + enterprise slotignored
    assert!(faf.contains("backend: slotignored"));
    assert!(faf.contains("admin: slotignored"));
}

#[test]
fn test_whisper_cpp_library_no_owner_field() {
    let readme = r#"
# whisper.cpp

High-performance inference of OpenAI's Whisper ASR model.

## Usage

To build the main program, run `make`.
"#;

    let faf = generate_faf(
        "whisper.cpp".to_string(),
        "ggml-org".to_string(),
        Some("High-performance inference of OpenAI's Whisper ASR model".to_string()),
        Some(readme.to_string()),
        None, // C++ project, no dep file
        Some("C++".to_string()),
    )
    .expect("generation should succeed");

    assert!(faf.contains("name: whisper.cpp"));
    // owner is no longer a top-level field — it lives under metadata.repository
    assert!(!faf.contains("owner: ggml-org"), "no legacy owner: field");
    assert!(
        faf.contains("repository: https://github.com/ggml-org/whisper.cpp"),
        "owner folded into metadata.repository"
    );
    assert!(faf.contains("type: library"));
    assert!(faf.contains("who: ggml-org team"));
    assert!(faf.contains("what: whisper.cpp"));
    assert!(faf.contains("how: \"See README: Usage\""));
    assert!(!faf.contains("Unknown"));
}
