import { redirect } from "next/navigation";

import { isPrivateModeActiveFromEnv } from "@/lib/site-auth";

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

  const params = await searchParams;
  const nextPath = sanitizeNextPath(params.next);

  return <SiteLoginForm nextPath={nextPath} />;
}

function sanitizeNextPath(raw?: string): string {
  const value = raw?.trim() || "/";
  if (!value.startsWith("/") || value.startsWith("/login")) {
    return "/";
  }

  return value;
}
