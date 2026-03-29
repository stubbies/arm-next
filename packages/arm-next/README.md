# ARM-Next: Agent-Ready Markdown

It automatically transforms your web pages into LLM-optimized Markdown and serves them to AI agents via content negotiation (`Accept: text/markdown`).

## Features

- **Rust/WASM Engine:** Blazing fast HTML-to-Markdown conversion.
- **Agentic Directives:** Control what AI sees using data-ai-ignore and data-ai-meta
- **Lazy Generation:** Convert pages on-demand and cache them at the CDN edge.

## Install

```bash
npm install arm-next
```

## Use in your Next.js app

1. **Config** — use `withArmNext`:
  ```ts
   // next.config.ts
   import type { NextConfig } from "next";
   import { withArmNext } from "arm-next/next/with-arm-next";

   export default withArmNext(nextConfig);
  ```

2. **Proxy** : Wrap your existing handler (ARM-Next runs first for `Accept: text/markdown` / agent UAs):

  ```ts
  // src/proxy.ts
  import { NextResponse } from "next/server";
  import { withArmNextProxy } from "arm-next/proxy";

  export default withArmNextProxy(async (req) => {
    // your routing, auth, header rewrites…
    return NextResponse.next();
  });
  ```


3. **Create the Route Handler**

Create a catch-all route to handle the conversion. Choose the runtime that fits your needs.

**Option A: Node.js (Recommended for Caching)**

Uses `unstable_cache` to prevent redundant conversions.

```ts
// app/api/markdown/[[...slug]]/route.ts
export { GET } from "arm-next/routes/markdown";
```

**Option B: Edge (Recommended for Low Latency)**

```ts
// app/api/markdown/[[...slug]]/route.ts
export { GET } from "arm-next/routes/markdown-edge";
```

## Curating the AI Experience

ARM-Next gives you fine-grained control over your DOM via data attributes:

```html
<!-- This will be stripped from the Markdown (ads, nav, etc.) -->
<nav data-ai-ignore>...</nav>

<!-- Add custom context to the Markdown Frontmatter -->
<span data-ai-meta="author">Jane Doe</span>
```

## Example response

````
---
author: John Doe
description: Sed ut perspiciatis voluptatem accusantium
keywords: similique, sunt, culpa, qui, officia, deserunt
title: Voluptatibus maiores alias doloribus asperiores repellat
---


# Introduction

Nam libero tempore, cum soluta nobis est eligendi optio cumque nihil impedit quo minus id quod maxime placeat facere possimus, omnis voluptas assumenda est, omnis dolor repellendus.
...
````

## Environment

- `ARM_MAX_TOKENS` — default `4096`
- `ARM_CONTINUED_MESSAGE` — truncation suffix
- `ARM_CONVERSION_OPTIONS_JSON` — `html-to-markdown-rs` options JSON
- `ARM_EXTRA_AGENT_UA_SUBSTRINGS` — extra agent `User-Agent` substrings

## Development

This repository is an npm workspace. The example app is `examples/minimal`, linked with `"arm-next": "*"` (resolved to the local workspace package).

From the monorepo root:

```bash
npm install
npm run build -w arm-next           # compile `packages/arm-next/dist/`
npm run build:wasm -w arm-next      # wasm-pack bundler → `packages/arm-next/wasm/bundler/`
npm run dev                         # runs the example app
```

### Rust tests

From the monorepo root (Rust toolchain required):

```bash
npm run test:rust          # default features (tiktoken)
npm run test:rust:wasm     # same tests + no-tiktoken truncation path (WASM-style)
```