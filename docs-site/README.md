# docs-site

Integration-first documentation site for `defi-tracker-lifecycle`.

The site is a small React + Vite application that consumes the crate’s WASM bindings for protocol
metadata, lifecycle transitions, and variant lookup helpers.

## Local Development

```bash
cd docs-site
npm install
npm run dev
```

`npm run dev` regenerates the WASM bindings first, then starts Vite.

## Useful Commands

```bash
cd docs-site
npm run wasm         # rebuild docs-site/src/wasm-pkg from the Rust crate
npm run lint         # eslint
npm run build        # regenerate WASM, type-check, and build the static site
```

## WASM Output

The generated bindings live in `docs-site/src/wasm-pkg/`.

- They are build artifacts, not hand-edited source files.
- The folder is gitignored.
- If you change `src/wasm.rs`, regenerate the bindings before running the site or building it.

## What The Site Covers

- Quick-start integration flow for the crate
- Lifecycle playground powered by the crate’s transition logic
- Static raw-variant lookup for protocol instruction and event names
- Protocol-specific integration notes for DCA, Jupiter Limit v1/v2, and Kamino
- Reliability/test-layer overview
