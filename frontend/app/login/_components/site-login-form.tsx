"use client";

import { FormEvent, useState } from "react";
import { useRouter } from "next/navigation";

type SiteLoginFormProps = {
  nextPath: string;
  labels: {
    title: string;
    description: string;
    passwordPlaceholder: string;
    pending: string;
    submit: string;
    failed: string;
  };
};

export function SiteLoginForm({ nextPath, labels }: SiteLoginFormProps) {
  const router = useRouter();
  const [password, setPassword] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    setError(null);

    try {
      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify({ password }),
      });

      if (!response.ok) {
        const data = (await response.json().catch(() => null)) as
          | { message?: string }
          | null;
        throw new Error(data?.message || labels.failed);
      }

      router.replace(nextPath);
      router.refresh();
    } catch (err) {
      const message = err instanceof Error ? err.message : labels.failed;
      setError(message);
      setPending(false);
    }
  }

  return (
    <section className="w-full max-w-md border-4 border-ink bg-paper p-8 shadow-hard-sm">
      <h1 className="font-headline text-4xl font-black uppercase">{labels.title}</h1>
      <p className="mt-3 text-sm text-ink-muted">{labels.description}</p>

      <form className="mt-6 space-y-4" onSubmit={onSubmit}>
        <input
          type="password"
          required
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          placeholder={labels.passwordPlaceholder}
          className="w-full border-2 border-ink bg-paper px-3 py-2 font-headline text-sm font-bold uppercase outline-none"
        />

        {error ? <p className="text-sm font-semibold text-accent-red">{error}</p> : null}

        <button
          disabled={pending}
          className="bauhaus-shadow-sm bauhaus-press inline-flex w-full items-center justify-center border-2 border-ink bg-accent-yellow px-4 py-2 font-headline text-sm font-black uppercase transition-all hover:bg-ink hover:text-paper disabled:opacity-50"
        >
          {pending ? labels.pending : labels.submit}
        </button>
      </form>
    </section>
  );
}
