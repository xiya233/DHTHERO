import { AdminLoginForm } from "./_components/admin-login-form";

type AdminLoginPageProps = {
  searchParams: Promise<{
    next?: string;
  }>;
};

export default async function AdminLoginPage({ searchParams }: AdminLoginPageProps) {
  const params = await searchParams;
  const nextPath = params.next?.trim() || "/admin";

  return <AdminLoginForm nextPath={nextPath.startsWith("/") ? nextPath : "/admin"} />;
}
