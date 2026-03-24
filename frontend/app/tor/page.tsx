export default function TorPage() {
  return (
    <section className="mx-auto max-w-3xl border-4 border-ink bg-paper p-8 shadow-hard-sm">
      <h1 className="font-headline text-5xl font-black uppercase">Tor</h1>
      <p className="mt-4 text-sm leading-relaxed text-ink-muted">
        This project currently keeps Tor as a static frontend entry by design. No backend Tor
        proxy/search API is enabled.
      </p>
    </section>
  );
}
