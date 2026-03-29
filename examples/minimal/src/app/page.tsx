export default function Home() {
  return (
    <main className="mx-auto flex max-w-xl flex-col gap-4 px-6 py-16">
      <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-100">
        arm-next example
      </h1>
      <p className="text-zinc-600 dark:text-zinc-400">
        Install <code className="rounded bg-zinc-100 px-1 dark:bg-zinc-900">arm-next</code>{" "}
        in your Next app, wire proxy and API routes, then request this page with{" "}
        <code className="rounded bg-zinc-100 px-1 dark:bg-zinc-900">Accept: text/markdown</code>.
      </p>
    </main>
  );
}
