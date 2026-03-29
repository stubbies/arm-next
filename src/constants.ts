/** Internal fetch header: skip markdown rewrite / recursion. */
export const ARM_INTERNAL_HEADER = "x-arm-internal";
export const ARM_INTERNAL_VALUE = "1";

/**
 * Default `Cache-Control` for HTML→Markdown page responses (CDN-friendly, 1h edge TTL).
 * Aligns with `unstable_cache` revalidate (3600s) in the Node markdown route.
 * Override with env `ARM_MARKDOWN_CACHE_CONTROL`.
 */
export const ARM_MARKDOWN_DEFAULT_CACHE_CONTROL =
  "public, s-maxage=3600, stale-while-revalidate=86400";
