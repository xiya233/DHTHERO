import { getCopy } from "@/lib/i18n";
import { getServerSitePreferences } from "@/lib/site-preferences-server";

export default async function TorPage() {
  const { locale } = await getServerSitePreferences();
  const copy = getCopy(locale);

  return (
    <section className="mx-auto max-w-3xl border-4 border-ink bg-paper p-8 shadow-hard-sm">
      <h1 className="font-headline text-5xl font-black uppercase">{copy.tor.title}</h1>
      <p className="mt-4 text-sm leading-relaxed text-ink-muted">{copy.tor.description}</p>
    </section>
  );
}
