import { cookies } from "next/headers";
import { NextResponse } from "next/server";

import { ADMIN_SESSION_COOKIE, getAdminPasswordFromEnv } from "@/lib/admin-auth";
import { getSessionSecretFromEnv, verifySessionToken } from "@/lib/session-token";
import { getPrivateSitePasswordFromEnv, isPrivateModeActiveFromEnv } from "@/lib/site-auth";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL?.replace(/\/$/, "") || "http://localhost:8080";

export async function GET() {
  const configuredPassword = getAdminPasswordFromEnv();
  const sessionSecret = getSessionSecretFromEnv();
  const privateModeActive = isPrivateModeActiveFromEnv();
  const sitePassword = getPrivateSitePasswordFromEnv();
  if (!configuredPassword) {
    return NextResponse.json(
      { code: "ADMIN_DISABLED", message: "admin dashboard password is not configured" },
      { status: 503 },
    );
  }
  if (!sessionSecret) {
    return NextResponse.json(
      { code: "SESSION_DISABLED", message: "session secret is not configured" },
      { status: 503 },
    );
  }

  const cookieStore = await cookies();
  const session = cookieStore.get(ADMIN_SESSION_COOKIE)?.value ?? "";
  const authorized = await verifySessionToken(session, "admin", sessionSecret);
  if (!authorized) {
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "unauthorized" },
      { status: 401 },
    );
  }

  let response: Response;
  try {
    const headers: Record<string, string> = {
      Accept: "application/json",
      "x-admin-password": configuredPassword,
    };
    if (privateModeActive) {
      headers["x-site-password"] = sitePassword;
    }

    response = await fetch(`${API_BASE_URL}/api/v1/admin/dashboard`, {
      cache: "no-store",
      headers,
    });
  } catch {
    return NextResponse.json(
      { code: "UPSTREAM_ERROR", message: "failed to request backend dashboard" },
      { status: 502 },
    );
  }

  const text = await response.text();
  const contentType = response.headers.get("content-type") || "application/json";
  return new NextResponse(text, {
    status: response.status,
    headers: {
      "content-type": contentType,
      "cache-control": "no-store",
    },
  });
}
