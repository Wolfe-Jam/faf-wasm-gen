# faf-generator-wasm

**Rust WASM Generator for FAF (Foundational AI-context Format)**

Generate `project.faf` files in the browser or at the edge.

---

## Mission

Build a Rust WASM generator that matches faf-cli v4.2.1 output exactly.

**Spec:** See `SPEC.md` for complete implementation specification.

**Bible:** `/Users/wolfejam/FAF/cli` (faf-cli v4.2.1)

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

```bash
wasm-pack build --target web --release
```

**Output:**
- `pkg/faf_generator_wasm_bg.wasm` (WASM binary)
- `pkg/faf_generator_wasm.js` (JS glue code)

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
- grok-1 (Python/JAX, ml-research type)
- test-faf-demo (JavaScript/React)
- faf-cli (TypeScript CLI)

---

## Success Criteria

✅ Generates identical .faf to faf-cli for same inputs
✅ Supports ml-research type (NEW - critical for Grok-1)
✅ Omits undetected fields (no "Unknown" hardcoding)
✅ Scores identically to faf-cli
✅ Passes WJTTC test suite

---

## Status

**Current:** Spec ready, awaiting implementation
**Owner:** To be implemented
**Timeline:** TBD

---

## Links

- [SPEC.md](./SPEC.md) - Complete implementation specification
- [faf-cli](../cli/) - Reference implementation (the bible)
- [xai-faf-zig](../xai-faf-zig/) - Zig WASM scorer (2.7KB)

---

**Built with 🦀 Rust + WASM for the edge**
