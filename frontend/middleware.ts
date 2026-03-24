import { NextRequest, NextResponse } from "next/server";

import { ADMIN_SESSION_COOKIE } from "@/lib/admin-auth";

function isProtectedPath(pathname: string): boolean {
  return pathname.startsWith("/admin") || pathname.startsWith("/api/admin/dashboard");
}

function isPublicAdminPath(pathname: string): boolean {
  return (
    pathname === "/admin/login" ||
    pathname.startsWith("/api/admin/login") ||
    pathname.startsWith("/api/admin/logout")
  );
}

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  if (!isProtectedPath(pathname) || isPublicAdminPath(pathname)) {
    return NextResponse.next();
  }

  const configuredPassword = process.env.ADMIN_DASHBOARD_PASSWORD?.trim();
  if (!configuredPassword) {
    if (pathname.startsWith("/api/")) {
      return NextResponse.json(
        { code: "ADMIN_DISABLED", message: "admin dashboard password is not configured" },
        { status: 503 },
      );
    }

    return NextResponse.redirect(new URL("/", request.url));
  }

  const session = request.cookies.get(ADMIN_SESSION_COOKIE)?.value ?? "";
  if (session === configuredPassword) {
    return NextResponse.next();
  }

  if (pathname.startsWith("/api/")) {
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "unauthorized" },
      { status: 401 },
    );
  }

  const loginUrl = new URL("/admin/login", request.url);
  loginUrl.searchParams.set("next", pathname);
  return NextResponse.redirect(loginUrl);
}

export const config = {
  matcher: ["/admin/:path*", "/api/admin/:path*"],
};
