export type ConvertInput = {
  html: string;
  conversionOptionsJson?: string | null;
  maxTokens?: number | null;
  continuedMessage?: string | null;
};

export type ConvertOutput = {
  markdown: string;
  tokenCount: number;
  truncated: boolean;
};

type WasmMod = typeof import("arm-next/wasm");

let wasmModulePromise: Promise<WasmMod> | null = null;

function loadWasmModule(): Promise<WasmMod> {
  if (!wasmModulePromise) {
    wasmModulePromise = import("arm-next/wasm");
  }
  return wasmModulePromise;
}

/** wasm-pack **bundler** output; dynamic import gives Webpack an async boundary before `.wasm`. */
export async function convertPageToMarkdown(
  input: ConvertInput,
): Promise<ConvertOutput> {
  const { convert_page_to_markdown_json } = await loadWasmModule();
  const jsonStr = convert_page_to_markdown_json(
    input.html,
    input.conversionOptionsJson ?? undefined,
    input.maxTokens ?? undefined,
    input.continuedMessage ?? undefined,
  );
  const parsed = JSON.parse(jsonStr) as {
    markdown: string;
    token_count: number;
    truncated: boolean;
  };
  return {
    markdown: parsed.markdown,
    tokenCount: parsed.token_count,
    truncated: parsed.truncated,
  };
}
