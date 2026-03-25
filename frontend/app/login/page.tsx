import { redirect } from "next/navigation";

import { getCopy } from "@/lib/i18n";
import { isPrivateModeActiveFromEnv } from "@/lib/site-auth";
import { getServerSitePreferences } from "@/lib/site-preferences-server";

import { SiteLoginForm } from "./_components/site-login-form";

type LoginPageProps = {
  searchParams: Promise<{
    next?: string;
  }>;
};

export default async function LoginPage({ searchParams }: LoginPageProps) {
  if (!isPrivateModeActiveFromEnv()) {
    redirect("/");
  }
  const { locale } = await getServerSitePreferences();
  const copy = getCopy(locale);

  const params = await searchParams;
  const nextPath = sanitizeNextPath(params.next);

  return (
    <SiteLoginForm
      nextPath={nextPath}
      labels={{
        title: copy.login.privateTitle,
        description: copy.login.privateDesc,
        passwordPlaceholder: copy.login.passwordPlaceholder,
        pending: copy.login.unlocking,
        submit: copy.login.unlock,
        failed: copy.login.failed,
      }}
    />
  );
}

function sanitizeNextPath(raw?: string): string {
  const value = raw?.trim() || "/";
  if (!value.startsWith("/") || value.startsWith("/login")) {
    return "/";
  }

  return value;
}
