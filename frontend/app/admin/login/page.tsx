import { getCopy } from "@/lib/i18n";
import { getServerSitePreferences } from "@/lib/site-preferences-server";

import { AdminLoginForm } from "./_components/admin-login-form";

type AdminLoginPageProps = {
  searchParams: Promise<{
    next?: string;
  }>;
};

export default async function AdminLoginPage({ searchParams }: AdminLoginPageProps) {
  const { locale } = await getServerSitePreferences();
  const copy = getCopy(locale);
  const params = await searchParams;
  const nextPath = params.next?.trim() || "/admin";

  return (
    <AdminLoginForm
      nextPath={nextPath.startsWith("/") ? nextPath : "/admin"}
      labels={{
        title: copy.login.adminTitle,
        description: copy.login.adminDesc,
        passwordPlaceholder: copy.login.passwordPlaceholder,
        pending: copy.login.signingIn,
        submit: copy.login.signIn,
        failed: copy.login.failed,
      }}
    />
  );
}
