import { convertPageToMarkdown } from "../wasm-runtime.js";
import {
  ARM_INTERNAL_HEADER,
  ARM_INTERNAL_VALUE,
  ARM_MARKDOWN_DEFAULT_CACHE_CONTROL,
} from "../constants.js";
import { unstable_cache } from "next/cache";
import { type NextRequest, NextResponse } from "next/server";

/**
 * HTML source: same-origin `fetch` of the page. Internal header
 * `x-arm-internal` avoids proxy rewrite recursion.
 */
export const runtime = "nodejs";

function safePath(path: string | null): string | null {
  if (!path || !path.startsWith("/") || path.includes("..") || path.includes("\0")) {
    return null;
  }
  return path;
}

function defaultMaxTokens(): number | undefined {
  const raw = process.env.ARM_MAX_TOKENS;
  if (raw === undefined || raw === "") {
    return 4096;
  }
  const n = Number(raw);
  return Number.isFinite(n) && n > 0 ? n : undefined;
}

async function fetchHtml(origin: string, pathname: string): Promise<string> {
  const url = `${origin}${pathname}`;
  const res = await fetch(url, {
    headers: {
      accept: "text/html,application/xhtml+xml",
      [ARM_INTERNAL_HEADER]: ARM_INTERNAL_VALUE,
    },
    next: { revalidate: 60 },
  } as RequestInit & { next?: { revalidate?: number } });
  if (!res.ok) {
    throw new Error(`Failed to fetch HTML: ${res.status} ${url}`);
  }
  return res.text();
}

export async function GET(request: NextRequest) {
  const path =
    safePath(request.nextUrl.searchParams.get("path")) ??
    safePath(request.headers.get("x-arm-original-path"));

  if (!path) {
    return NextResponse.json({ error: "Missing or invalid path" }, { status: 400 });
  }

  const origin = request.nextUrl.origin;
  const maxTokens = defaultMaxTokens();
  const continued =
    process.env.ARM_CONTINUED_MESSAGE ?? "[CONTINUED IN NEXT CHUNK]";

  const run = unstable_cache(
    async () => {
      const html = await fetchHtml(origin, path);
      return await convertPageToMarkdown({
        html,
        maxTokens,
        continuedMessage: continued,
        conversionOptionsJson: process.env.ARM_CONVERSION_OPTIONS_JSON,
      });
    },
    ["arm-next-md", path, String(maxTokens ?? ""), continued],
    { revalidate: 3600, tags: [`arm-md:${path}`] },
  );

  try {
    const out = await run();
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
    const message = e instanceof Error ? e.message : "Conversion failed";
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
