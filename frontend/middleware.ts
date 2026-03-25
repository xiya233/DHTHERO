import { NextRequest, NextResponse } from "next/server";

import { ADMIN_SESSION_COOKIE } from "@/lib/admin-auth";
import {
  getSessionSecretFromEnv,
  verifySessionToken,
} from "@/lib/session-token";
import {
  SITE_SESSION_COOKIE,
  isPrivateModeActiveFromEnv,
} from "@/lib/site-auth";

function isApiPath(pathname: string): boolean {
  return pathname.startsWith("/api/");
}

function isStaticAssetPath(pathname: string): boolean {
  return (
    pathname.startsWith("/_next/") ||
    pathname === "/favicon.ico" ||
    pathname === "/robots.txt" ||
    pathname === "/sitemap.xml" ||
    /\.[^/]+$/.test(pathname)
  );
}

function isPrivatePublicPath(pathname: string): boolean {
  return (
    pathname === "/login" ||
    pathname.startsWith("/api/auth/login") ||
    pathname.startsWith("/api/auth/logout")
  );
}

function isAdminProtectedPath(pathname: string): boolean {
  return pathname.startsWith("/admin") || pathname.startsWith("/api/admin/");
}

function isAdminPublicPath(pathname: string): boolean {
  return (
    pathname === "/admin/login" ||
    pathname.startsWith("/api/admin/login") ||
    pathname.startsWith("/api/admin/logout")
  );
}

function sanitizeRedirectPath(pathname: string | null): string {
  const value = pathname?.trim() || "/";
  if (!value.startsWith("/") || value.startsWith("/login")) {
    return "/";
  }

  return value;
}

export async function middleware(request: NextRequest) {
  const { pathname, search } = request.nextUrl;
  if (isStaticAssetPath(pathname)) {
    return NextResponse.next();
  }

  const sessionSecret = getSessionSecretFromEnv();

  if (isPrivateModeActiveFromEnv()) {
    if (!sessionSecret) {
      if (isApiPath(pathname)) {
        return NextResponse.json(
          { code: "SESSION_DISABLED", message: "session secret is not configured" },
          { status: 503 },
        );
      }

      return new NextResponse("session secret is not configured", { status: 503 });
    }

    const siteSession = request.cookies.get(SITE_SESSION_COOKIE)?.value ?? "";
    const siteAuthorized = await verifySessionToken(siteSession, "site", sessionSecret);

    if (pathname === "/login" && siteAuthorized) {
      const nextPath = sanitizeRedirectPath(request.nextUrl.searchParams.get("next"));
      return NextResponse.redirect(new URL(nextPath, request.url));
    }

    if (!isPrivatePublicPath(pathname) && !siteAuthorized) {
      if (isApiPath(pathname)) {
        return NextResponse.json(
          { code: "UNAUTHORIZED", message: "site is in private mode" },
          { status: 401 },
        );
      }

      const loginUrl = new URL("/login", request.url);
      loginUrl.searchParams.set("next", `${pathname}${search}`);
      return NextResponse.redirect(loginUrl);
    }
  }

  if (!isAdminProtectedPath(pathname) || isAdminPublicPath(pathname)) {
    return NextResponse.next();
  }

  const configuredAdminPassword = process.env.ADMIN_DASHBOARD_PASSWORD?.trim();
  if (!configuredAdminPassword) {
    if (isApiPath(pathname)) {
      return NextResponse.json(
        { code: "ADMIN_DISABLED", message: "admin dashboard password is not configured" },
        { status: 503 },
      );
    }

    return NextResponse.redirect(new URL("/", request.url));
  }

  if (!sessionSecret) {
    if (isApiPath(pathname)) {
      return NextResponse.json(
        { code: "SESSION_DISABLED", message: "session secret is not configured" },
        { status: 503 },
      );
    }

    return new NextResponse("session secret is not configured", { status: 503 });
  }

  const adminSession = request.cookies.get(ADMIN_SESSION_COOKIE)?.value ?? "";
  const adminAuthorized = await verifySessionToken(adminSession, "admin", sessionSecret);
  if (adminAuthorized) {
    return NextResponse.next();
  }

  if (isApiPath(pathname)) {
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "unauthorized" },
      { status: 401 },
    );
  }

  const loginUrl = new URL("/admin/login", request.url);
  loginUrl.searchParams.set("next", `${pathname}${search}`);
  return NextResponse.redirect(loginUrl);
}

export const config = {
  matcher: ["/:path*"],
};
