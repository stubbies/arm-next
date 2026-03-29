# arm-next

Monorepo for **arm-next** — Rust-powered HTML→Markdown for Next.js (WebAssembly, Edge-compatible).

- **Published package:** [`packages/arm-next`](./packages/arm-next) (`npm install arm-next`)
- **Example app:** [`examples/minimal`](./examples/minimal)

Documentation and feature overview live in the [package README](./packages/arm-next/README.md).

## Development

```bash
npm install
npm run build        # build the library (TS + optional: build:wasm inside package)
npm run dev          # example Next.js app
```

Rust tests:

```bash
npm run test:rust
npm run test:rust:wasm
```

Releases use [Changesets](https://github.com/changesets/changesets): add a changeset from the repo root, merge the “Version Packages” PR, then CI publishes to npm.

## License

MIT — see [LICENSE](./LICENSE).
