import { cookies } from "next/headers";
import { NextResponse } from "next/server";

import { ADMIN_SESSION_COOKIE, getAdminPasswordFromEnv } from "@/lib/admin-auth";
import { getPrivateSitePasswordFromEnv, isPrivateModeActiveFromEnv } from "@/lib/site-auth";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL?.replace(/\/$/, "") || "http://localhost:8080";

async function resolveAuthorizedHeaders(): Promise<
  { ok: true; headers: Record<string, string> } | { ok: false; response: NextResponse }
> {
  const configuredPassword = getAdminPasswordFromEnv();
  if (!configuredPassword) {
    return {
      ok: false,
      response: NextResponse.json(
        { code: "ADMIN_DISABLED", message: "admin dashboard password is not configured" },
        { status: 503 },
      ),
    };
  }

  const cookieStore = await cookies();
  const session = cookieStore.get(ADMIN_SESSION_COOKIE)?.value ?? "";
  if (session !== configuredPassword) {
    return {
      ok: false,
      response: NextResponse.json(
        { code: "UNAUTHORIZED", message: "unauthorized" },
        { status: 401 },
      ),
    };
  }

  const headers: Record<string, string> = {
    Accept: "application/json",
    "x-admin-password": configuredPassword,
  };

  if (isPrivateModeActiveFromEnv()) {
    headers["x-site-password"] = getPrivateSitePasswordFromEnv();
  }

  return { ok: true, headers };
}

export async function GET() {
  const auth = await resolveAuthorizedHeaders();
  if (!auth.ok) {
    return auth.response;
  }

  let response: Response;
  try {
    response = await fetch(`${API_BASE_URL}/api/v1/admin/site-settings`, {
      cache: "no-store",
      headers: auth.headers,
    });
  } catch {
    return NextResponse.json(
      { code: "UPSTREAM_ERROR", message: "failed to request backend site settings" },
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

export async function PUT(request: Request) {
  const auth = await resolveAuthorizedHeaders();
  if (!auth.ok) {
    return auth.response;
  }

  const payload = await request.text();
  let response: Response;
  try {
    response = await fetch(`${API_BASE_URL}/api/v1/admin/site-settings`, {
      method: "PUT",
      cache: "no-store",
      headers: {
        ...auth.headers,
        "content-type": "application/json",
      },
      body: payload,
    });
  } catch {
    return NextResponse.json(
      { code: "UPSTREAM_ERROR", message: "failed to request backend site settings" },
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
