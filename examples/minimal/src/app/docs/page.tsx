export default function DocsPage() {
  return (
    <main className="mx-auto max-w-2xl px-6 py-16">
      <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-100">
        Docs
      </h1>
      <p className="mt-4 text-zinc-600 dark:text-zinc-400">
        Included in Markdown for agents.
      </p>
      <aside
        className="mt-8 rounded-lg border border-zinc-200 p-4 text-sm dark:border-zinc-800"
        data-ai-ignore
      >
        Sidebar pruned via <code data-ai-ignore>data-ai-ignore</code> in Rust.
      </aside>
    </main>
  );
}
