import {
  type NextRequest,
  NextResponse,
  type NextProxy,
} from "next/server";
import { ARM_INTERNAL_HEADER, ARM_INTERNAL_VALUE } from "./constants.js";

export type ArmNextProxyOptions = {
  /** Default: `/api/ai/markdown` */
  markdownApiPath?: string;
};

function wantsMarkdown(request: NextRequest): boolean {
  if (request.headers.get(ARM_INTERNAL_HEADER) === ARM_INTERNAL_VALUE) {
    return false;
  }
  const accept = request.headers.get("accept") ?? "";
  if (accept.includes("text/markdown")) {
    return true;
  }
  const ua = request.headers.get("user-agent") ?? "";
  if (ua.includes("AI-Agent")) {
    return true;
  }
  const extra = process.env.ARM_EXTRA_AGENT_UA_SUBSTRINGS?.split(",") ?? [];
  for (const s of extra) {
    const t = s.trim();
    if (t && ua.includes(t)) {
      return true;
    }
  }
  return false;
}

/**
 * Rewrite HTML page requests into the Markdown API route when clients ask for Markdown or match agent UA rules.
 */
export function createArmNextProxy(
  options: ArmNextProxyOptions = {},
): NextProxy {
  const markdownApiPath = options.markdownApiPath ?? "/api/ai/markdown";

  return function armNextProxy(request: NextRequest) {
    const { pathname } = request.nextUrl;
    if (
      pathname.startsWith("/api/") ||
      pathname.startsWith("/_next") ||
      pathname.startsWith("/.well-known/") ||
      pathname.includes(".")
    ) {
      return NextResponse.next();
    }

    if (!wantsMarkdown(request)) {
      return NextResponse.next();
    }

    const url = request.nextUrl.clone();
    url.pathname = markdownApiPath;
    url.searchParams.set("path", pathname);
    const requestHeaders = new Headers(request.headers);
    requestHeaders.set("x-arm-original-path", pathname);

    const res = NextResponse.rewrite(url, {
      request: { headers: requestHeaders },
    });
    res.headers.set("vary", "accept, user-agent");
    return res;
  };
}
