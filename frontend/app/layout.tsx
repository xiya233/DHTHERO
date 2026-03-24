import type { Metadata } from "next";
import Link from "next/link";
import { Inter, Space_Grotesk } from "next/font/google";

import "./globals.css";
import { getFeatures } from "@/lib/api";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

const spaceGrotesk = Space_Grotesk({
  variable: "--font-space-grotesk",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "DHT_MAGNET",
  description: "Bauhaus inspired DHT magnet search engine",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const features = await getFeatures();

  return (
    <html
      lang="en"
      className={`${inter.variable} ${spaceGrotesk.variable} h-full antialiased`}
    >
      <body className="min-h-full bg-paper text-ink">
        <div className="min-h-screen grid-paper">
          <header className="border-b-4 border-ink bg-paper px-6 py-4">
            <div className="mx-auto flex w-full max-w-6xl items-center justify-between gap-6">
              <Link
                href="/"
                className="border-2 border-ink px-3 py-1 font-headline text-2xl font-black uppercase tracking-tight"
              >
                DHT_MAGNET
              </Link>

              <nav className="hidden items-center gap-6 font-headline text-sm font-bold uppercase tracking-widest md:flex">
                {features.latest_enabled && (
                  <Link href="/latest" className="hover:underline">
                    Latest
                  </Link>
                )}
                {features.trending_enabled && (
                  <Link href="/trending" className="hover:underline">
                    Trending
                  </Link>
                )}
                <Link href="/tor" className="hover:underline">
                  Tor
                </Link>
              </nav>
            </div>
          </header>

          <main className="mx-auto w-full max-w-6xl px-6 py-10 pb-24 md:pb-12">{children}</main>

          <footer className="border-t-4 border-ink bg-ink px-6 py-8 text-paper">
            <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 text-sm font-headline uppercase tracking-wider md:flex-row md:items-center md:justify-between">
              <p>© {new Date().getFullYear()} DHT_BAUHAUS</p>
              <div className="flex items-center gap-6">
                {features.latest_enabled && <Link href="/latest">Latest</Link>}
                {features.trending_enabled && <Link href="/trending">Trending</Link>}
                <Link href="/tor">Tor</Link>
              </div>
            </div>
          </footer>

          <div className="fixed bottom-0 left-0 right-0 z-30 border-t-4 border-ink bg-paper px-4 py-2 md:hidden">
            <div className="flex items-center justify-around font-headline text-[11px] font-bold uppercase tracking-wider">
              <Link href="/">Home</Link>
              {features.latest_enabled && <Link href="/latest">Latest</Link>}
              {features.trending_enabled && <Link href="/trending">Hot</Link>}
              <Link href="/tor">Tor</Link>
            </div>
          </div>
        </div>
      </body>
    </html>
  );
}
