# Changelog

## 2.0.0 (2026-06-04) — faf-cli v6.8 parity + rename

First public release. Renamed from `faf-generator-wasm`; the generate sibling
of `faf-wasm-sdk`.

- Generate `project.faf` matching faf-cli v6.8: `faf_version 3.3`, 33-slot Mk4 model
- Canonical app types (`data-science`, `frontend`, `mcp`, `library`, …) with
  `slotignored` for slots outside the type's active categories
- Lean section set; dropped the legacy `ai_tldr`/`faf_dna`/`ai_scoring_details` blocks
- YAML-safe scalar emission (quotes values containing `": "`)
- Round-trips clean through the faf-cli scorer (mcp 17 / frontend 16 / library 12 active slots)
- 11 tests pass; builds to `wasm32-unknown-unknown`
