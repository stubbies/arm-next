import { convertPageToMarkdown } from "../wasm-runtime.js";
import {
  ARM_INTERNAL_HEADER,
  ARM_INTERNAL_VALUE,
  ARM_MARKDOWN_DEFAULT_CACHE_CONTROL,
} from "../constants.js";
import { type NextRequest, NextResponse } from "next/server";

export const runtime = "edge";

function safePath(path: string | null): string | null {
  if (!path || !path.startsWith("/") || path.includes("..") || path.includes("\0")) {
    return null;
  }
  return path;
}

function defaultMaxTokens(): number {
  const raw = process.env.ARM_MAX_TOKENS;
  if (raw === undefined || raw === "") {
    return 4096;
  }
  const n = Number(raw);
  return Number.isFinite(n) && n > 0 ? n : 4096;
}

/**
 * Same WASM as the Node route (`arm-next/wasm`, wasm-pack **bundler**); Next resolves it for Edge or Node.
 */
export async function GET(request: NextRequest) {
  const path =
    safePath(request.nextUrl.searchParams.get("path")) ??
    safePath(request.headers.get("x-arm-original-path"));
  if (!path) {
    return NextResponse.json({ error: "Missing or invalid path" }, { status: 400 });
  }

  const origin = request.nextUrl.origin;
  const htmlRes = await fetch(`${origin}${path}`, {
    headers: {
      accept: "text/html,application/xhtml+xml",
      [ARM_INTERNAL_HEADER]: ARM_INTERNAL_VALUE,
    },
  });
  if (!htmlRes.ok) {
    return NextResponse.json(
      { error: `HTML fetch failed: ${htmlRes.status}` },
      { status: htmlRes.status },
    );
  }
  const html = await htmlRes.text();

  try {
    const continued =
      process.env.ARM_CONTINUED_MESSAGE ?? "[CONTINUED IN NEXT CHUNK]";
    const out = await convertPageToMarkdown({
      html,
      maxTokens: defaultMaxTokens(),
      continuedMessage: continued,
      conversionOptionsJson: process.env.ARM_CONVERSION_OPTIONS_JSON,
    });
    const headers = new Headers();
    headers.set("content-type", "text/markdown; charset=utf-8");
    headers.set("x-ai-token-count", String(out.tokenCount));
    if (out.truncated) {
      headers.set("x-ai-truncated", "1");
    }
    headers.set("vary", "accept, user-agent");
    const cacheControl =
      process.env.ARM_MARKDOWN_CACHE_CONTROL?.trim() ||
      ARM_MARKDOWN_DEFAULT_CACHE_CONTROL;
    headers.set("cache-control", cacheControl);
    return new NextResponse(out.markdown, { status: 200, headers });
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    return NextResponse.json(
      { error: "Markdown conversion failed", detail: message },
      { status: 500 },
    );
  }
}
