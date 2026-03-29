import type { NextConfig } from "next";

export function withArmNext(config: NextConfig = {}): NextConfig {
  const existingHeaders = config.headers;
  return {
    ...config,
    ...(config.turbopack != null ? { turbopack: { ...config.turbopack } } : {}),
    async headers() {
      const arm = [
        {
          source: "/ai-md/:path*",
          headers: [
            { key: "Content-Type", value: "text/markdown; charset=utf-8" },
            { key: "Cache-Control", value: "public, max-age=3600" },
          ],
        },
      ] as NonNullable<Awaited<ReturnType<NonNullable<NextConfig["headers"]>>>>;
      const rest = existingHeaders ? await existingHeaders() : [];
      return [...arm, ...rest];
    },
  };
}
