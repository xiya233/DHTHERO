import type { Metadata } from "next";
import Link from "next/link";
import { Inter, Space_Grotesk } from "next/font/google";

import { BauhausIcon } from "@/components/bauhaus-icon";
import { getFeatures } from "@/lib/api";
import "./globals.css";

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
          <header className="top-0 z-50 w-full max-w-none border-b-4 border-ink bg-paper px-6 py-4">
            <div className="mx-auto flex w-full max-w-6xl items-center justify-between">
              <Link
                href="/"
                className="border-2 border-ink px-2 py-1 font-headline text-3xl font-black uppercase tracking-tighter"
              >
                DHT_MAGNET
              </Link>

              <nav className="hidden gap-8 font-headline text-sm font-bold uppercase tracking-tighter md:flex">
                {features.latest_enabled && (
                  <Link
                    href="/latest"
                    className="text-accent-yellow underline decoration-4 underline-offset-8 transition-colors duration-75 hover:bg-accent-yellow hover:text-ink"
                  >
                    Latest
                  </Link>
                )}
                {features.trending_enabled && (
                  <Link
                    href="/trending"
                    className="transition-colors duration-75 hover:bg-accent-yellow hover:text-ink"
                  >
                    Trending
                  </Link>
                )}
                <Link
                  href="/tor"
                  className="transition-colors duration-75 hover:bg-accent-yellow hover:text-ink"
                >
                  Tor
                </Link>
              </nav>

              <button className="bauhaus-press p-2 transition-all">
                <BauhausIcon name="dark_mode" className="size-6" />
              </button>
            </div>
          </header>

          <main className="mx-auto flex w-full max-w-6xl flex-col items-center justify-center px-6 py-12 pb-28 md:pb-12">
            {children}
          </main>

          <footer className="w-full border-t-4 border-ink bg-ink px-8 py-12 text-paper">
            <div className="mx-auto flex w-full max-w-6xl flex-col items-center justify-between gap-6 md:flex-row">
              <div className="font-headline text-sm font-bold uppercase tracking-widest">
                © {new Date().getFullYear()} DHT_BAUHAUS
              </div>
              <div className="flex gap-8 font-headline text-sm font-bold uppercase tracking-widest">
                {features.latest_enabled && (
                  <Link className="underline transition-all hover:text-accent-yellow" href="/latest">
                    Latest
                  </Link>
                )}
                {features.trending_enabled && (
                  <Link className="underline transition-all hover:text-accent-yellow" href="/trending">
                    Trending
                  </Link>
                )}
                <Link className="underline transition-all hover:text-accent-yellow" href="/tor">
                  Tor
                </Link>
                <Link className="underline transition-all hover:text-accent-yellow" href="/tor">
                  DMCA
                </Link>
              </div>
              <div className="flex gap-4">
                <div className="h-8 w-8 bg-accent-yellow" />
                <div className="h-8 w-8 bg-accent-red" />
                <div className="h-8 w-8 bg-accent-blue" />
              </div>
            </div>
          </footer>

          <div className="fixed bottom-0 left-0 right-0 z-40 border-t-4 border-ink bg-paper px-4 py-2 md:hidden">
            <div className="flex justify-around">
              <Link href="/" className="flex flex-col items-center p-2 text-accent-yellow">
                <BauhausIcon name="home" className="size-5" />
                <span className="font-headline text-[10px] font-bold uppercase">Home</span>
              </Link>
              {features.trending_enabled && (
                <Link href="/trending" className="flex flex-col items-center p-2 text-ink">
                  <BauhausIcon name="trending_up" className="size-5" />
                  <span className="font-headline text-[10px] font-bold uppercase">Hot</span>
                </Link>
              )}
              <Link href="/latest" className="flex flex-col items-center p-2 text-ink">
                <BauhausIcon name="database" className="size-5" />
                <span className="font-headline text-[10px] font-bold uppercase">DHT</span>
              </Link>
              <Link href="/tor" className="flex flex-col items-center p-2 text-ink">
                <BauhausIcon name="settings" className="size-5" />
                <span className="font-headline text-[10px] font-bold uppercase">Set</span>
              </Link>
            </div>
          </div>
        </div>
      </body>
    </html>
  );
}
