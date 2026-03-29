import {
  type NextFetchEvent,
  type NextRequest,
  NextResponse,
  type NextProxy,
} from "next/server";
import { ARM_INTERNAL_HEADER, ARM_INTERNAL_VALUE } from "./constants.js";

type NextProxyResult = Awaited<ReturnType<NextProxy>>;

export type ArmNextProxyOptions = {
  /** Default: `/api/ai/markdown` */
  markdownApiPath?: string;
};

export type WithArmNextProxyOptions = ArmNextProxyOptions;

export type ArmNextUserProxyHandler = (
  request: NextRequest,
  event?: NextFetchEvent,
) => NextProxyResult | Promise<NextProxyResult>;

function isNextResponse(res: NextProxyResult): res is NextResponse {
  return res instanceof NextResponse;
}

function armNextIssuedRewrite(res: NextProxyResult): boolean {
  return isNextResponse(res) && res.headers.has("x-middleware-rewrite");
}

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

  return function armNextProxy(request: NextRequest, _event: NextFetchEvent) {
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

/**
 * Compose existing `proxy.ts` handler with ARM-Next Markdown negotiation.
 *
 * @example
 * ```ts
 * export default withArmNextProxy(async (req) => {
 *   // host routing, auth, …
 *   return NextResponse.next();
 * });
 * ```
 */
export function withArmNextProxy(
  handler: ArmNextUserProxyHandler,
  options: WithArmNextProxyOptions = {},
): NextProxy {
  const armNext = createArmNextProxy(options);
  return async function composed(request: NextRequest, event: NextFetchEvent) {
    const armRes = await armNext(request, event);
    if (armNextIssuedRewrite(armRes)) {
      return armRes;
    }
    return handler(request, event);
  };
}
