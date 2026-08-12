# faf-wasm-gen

**Rust WASM Generator for FAF (Foundational AI-context Format)**

Generate `project.faf` files in the browser or at the edge.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-WASM-orange)](https://www.rust-lang.org/)
[![project.faf](https://img.shields.io/badge/project.faf-inside-008B8B)](https://github.com/Wolfe-Jam/faf)

⭐ Bookmarks it for you, helps other devs find it too.

---

## Mission

Build a Rust WASM generator that matches faf-cli v6.8 output exactly (faf_version `"3.3"`, 33-slot Mk4 model).

**Spec:** See `SPEC.md` for complete implementation specification.

**Bible:** `/Users/wolfejam/FAF/cli` (faf-cli v6.8) — `src/core/slots.ts`

---

## Deployment Targets

| Target | Platform | Purpose |
|--------|----------|---------|
| **builder.faf.one** | Vercel | Traditional .faf generation (browser + download/commit) |
| **MCPaaS.live** | Cloudflare Workers | MCP generation at edge (+ GitHub commit) |
| **FAF-Builder** | Vercel | Generic version of builder.faf.one |

---

## Architecture

**DOUBLE-WHAMMY:**
- **This repo (Rust WASM):** Generate .faf (211KB)
- **xai-faf-zig (Zig WASM):** Score .faf (2.7KB)

**Combined:** ~213KB total, sub-5ms generation + scoring

---

## Build

### Quick Build (if rustup is properly configured)

```bash
wasm-pack build --target web --release
```

### Alternative Build (rustup + Homebrew Rust conflict workaround)

If you have both rustup and Homebrew Rust installed on macOS:

```bash
# Build WASM binary
RUSTC=~/.rustup/toolchains/stable-x86_64-apple-darwin/bin/rustc \
  ~/.rustup/toolchains/stable-x86_64-apple-darwin/bin/cargo \
  build --target wasm32-unknown-unknown --release

# Generate JS glue code (requires wasm-bindgen-cli)
cargo install wasm-bindgen-cli
wasm-bindgen target/wasm32-unknown-unknown/release/faf_wasm_gen.wasm \
  --out-dir pkg --target web
```

**Output:**
- `pkg/faf_wasm_gen_bg.wasm` (WASM binary ~1.2MB)
- `pkg/faf_wasm_gen.js` (JS glue code)

**Optimization:**

Optimized binary included in `pkg/` (1.04MB final):

```bash
wasm-opt -Oz \
  --enable-mutable-globals \
  --enable-bulk-memory \
  --enable-nontrapping-float-to-int \
  --enable-sign-ext \
  --converge \
  --strip-debug --strip-dwarf --strip-producers \
  target/wasm32-unknown-unknown/release/faf_wasm_gen.wasm \
  -o pkg/faf_wasm_gen_bg.wasm
```

**Size Breakdown:**
- Original: 1.19MB
- Optimized: 1.04MB (11.9% reduction)
- regex: ~300KB (README pattern matching)
- chrono: ~200KB (timestamp generation)
- serde/serde_json: ~150KB (JSON parsing)
- Core logic: ~400KB

**Note:** 1.04MB is production-ready. Further reduction to 200-250KB would require removing dependencies and sacrificing features (regex extraction, JSON parsing, etc.).

---

## Deploy

**To builder.faf.one:**
```bash
cp pkg/*.wasm /Users/wolfejam/FAF/grok-faf-elite/static/
cp pkg/*.js   /Users/wolfejam/FAF/grok-faf-elite/static/
```

**To MCPaaS:**
```bash
cp pkg/*.wasm /Users/wolfejam/FAF/mcpaas-cf/src/
cp pkg/*.js   /Users/wolfejam/FAF/mcpaas-cf/src/
```

---

## Test

Must pass WJTTC test suite against faf-cli as ground truth:

```bash
cargo test
```

**Test repos:**
- grok-1 (Python/JAX, data-science type)
- test-faf-demo (JavaScript/React)
- faf-cli (TypeScript CLI)

---

## Success Criteria

✅ Generates current faf_version "3.3" / 33-slot Mk4 .faf
✅ Canonical app types (data-science, frontend, mcp, library, …)
✅ `slotignored` for slots outside the type's active categories
✅ Round-trips through faf-cli's scorer (mcp 17 / frontend 16 / library 12 active slots)
✅ No "Unknown" hardcoding; score computed on read (not stored)

---

## Status

**Implementation:** Complete ✅ (730 lines Rust)
**Tests:** 7/7 passing ✅ (4 unit + 3 integration)
**WASM Build:** Optimized ✅ (1.04MB final, 11.9% reduction from 1.19MB)
**Grok-1 Test:** 85% ◇ Bronze (handoff_ready: true)
**Owner:** wolfejam
**Date:** 2026-02-07

---

## Links

- [SPEC.md](./SPEC.md) - Complete implementation specification
- [faf-cli](../cli/) - Reference implementation (the bible)
- [xai-faf-zig](../xai-faf-zig/) - Zig WASM scorer (2.7KB)

---

**Built with 🦀 Rust + WASM for the edge**
