/*!
 * Integration tests for faf-generator-wasm
 * Tests with real repository data (grok-1)
 */

use faf_generator_wasm::generate_faf;

#[test]
fn test_grok1_ml_research_detection() {
    // Real Grok-1 README.md content (simplified)
    let readme = r#"
# Grok-1

This repository contains JAX example code for Grok-1 open-weights model.

## Model Architecture

Grok-1 is a 314B parameter Mixture-of-Experts model trained by xAI.

## Installation

```bash
pip install -r requirements.txt
```

## Usage

Run the model:

```bash
python run.py
```
"#;

    // Real Grok-1 pyproject.toml content
    let pyproject = r#"
[tool.poetry]
name = "grok-1"
version = "1.0.0"
description = "Grok-1 open release"

[tool.poetry.dependencies]
python = "^3.10"
jax = { version = "0.4.20", extras = ["cuda12_pip"] }
flax = "^0.7.5"
numpy = "^1.26.0"
sentencepiece = "^0.2.0"

[build-system]
requires = ["poetry-core"]
build-backend = "poetry.core.masonry.api"
"#;

    // Generate .faf
    let result = generate_faf(
        "grok-1".to_string(),
        "xai-org".to_string(),
        Some("Grok-1 open release".to_string()),
        Some(readme.to_string()),
        Some(pyproject.to_string()),
        Some("Python".to_string()),
    );

    assert!(result.is_ok(), "Generation should succeed");

    let faf = result.unwrap();

    // Verify critical fields
    assert!(faf.contains("type: ml-research"), "Should detect ml-research type");
    assert!(faf.contains("name: grok-1"), "Should have correct name");
    assert!(faf.contains("package_manager: poetry"), "Should detect poetry");
    assert!(faf.contains("runtime: Python"), "Should detect Python runtime");
    assert!(faf.contains("backend: Python"), "Should detect Python backend");

    // Verify 6 Ws extraction
    assert!(faf.contains("who: xai-org team"), "Should extract WHO");
    assert!(faf.contains("what: Grok-1"), "Should extract WHAT from title");
    assert!(faf.contains("where: GitHub"), "Should have WHERE");
    assert!(faf.contains("how: See README: Installation"), "Should detect Installation section");

    // Verify no Unknown hardcoding
    assert!(!faf.contains("Unknown"), "Should not contain Unknown");

    // Verify slot calculation (ml-research = 14 slots)
    assert!(faf.contains("total_slots: 14"), "Should have 14 slots for ml-research type");

    println!("\n=== Generated project.faf for Grok-1 ===\n{}\n", faf);
}

#[test]
fn test_react_app_detection() {
    let readme = r#"
# My React App

A modern React application.

## Getting Started

Install dependencies and start the dev server.
"#;

    let package_json = r#"{
  "name": "my-react-app",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}"#;

    let result = generate_faf(
        "my-react-app".to_string(),
        "test-org".to_string(),
        Some("A modern React application".to_string()),
        Some(readme.to_string()),
        Some(package_json.to_string()),
        Some("JavaScript".to_string()),
    );

    assert!(result.is_ok());
    let faf = result.unwrap();

    assert!(faf.contains("type: web-app"), "Should detect web-app type");
    assert!(faf.contains("frontend: React"), "Should detect React");
    assert!(faf.contains("build: Vite"), "Should detect Vite");
    assert!(faf.contains("runtime: Node.js"), "Should detect Node.js");
    assert!(faf.contains("package_manager: npm"), "Should detect npm");
    assert!(faf.contains("total_slots: 16"), "Should have 16 slots for web-app type");

    println!("\n=== Generated project.faf for React App ===\n{}\n", faf);
}

#[test]
fn test_minimal_repo_no_hardcoded_unknowns() {
    // Minimal repo with just a name
    let result = generate_faf(
        "minimal-repo".to_string(),
        "test-user".to_string(),
        None,
        None,
        None,
        Some("Rust".to_string()),
    );

    assert!(result.is_ok());
    let faf = result.unwrap();

    // Should have basic structure but NO "Unknown" values
    assert!(!faf.contains("Unknown"), "Should not contain any Unknown values");
    assert!(faf.contains("name: minimal-repo"), "Should have name");
    assert!(faf.contains("type: library"), "Should default to library type");

    // Stack section should be minimal or empty (no Unknown fields)
    // Verify it doesn't hardcode Unknown for backend, frontend, etc.

    println!("\n=== Generated project.faf for Minimal Repo ===\n{}\n", faf);
}
