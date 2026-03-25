import { cookies } from "next/headers";
import { NextResponse } from "next/server";

import {
  checkLoginRateLimit,
  clearLoginFailures,
  registerLoginFailure,
  resolveClientIpFromHeaders,
} from "@/lib/login-rate-limit";
import { getSessionSecretFromEnv, issueSessionToken } from "@/lib/session-token";
import {
  ADMIN_SESSION_COOKIE,
  ADMIN_SESSION_TTL_SECONDS,
  getAdminPasswordFromEnv,
} from "@/lib/admin-auth";

type LoginPayload = {
  password?: string;
};

const ADMIN_LOGIN_SCOPE = "admin_login";

export async function POST(request: Request) {
  const configuredPassword = getAdminPasswordFromEnv();
  if (!configuredPassword) {
    return NextResponse.json(
      { code: "ADMIN_DISABLED", message: "admin dashboard password is not configured" },
      { status: 503 },
    );
  }

  const sessionSecret = getSessionSecretFromEnv();
  if (!sessionSecret) {
    return NextResponse.json(
      { code: "SESSION_DISABLED", message: "session secret is not configured" },
      { status: 503 },
    );
  }

  const clientIp = resolveClientIpFromHeaders(request.headers);
  const rateLimit = checkLoginRateLimit(ADMIN_LOGIN_SCOPE, clientIp);
  if (rateLimit.limited) {
    return NextResponse.json(
      { code: "RATE_LIMITED", message: "too many failed login attempts" },
      {
        status: 429,
        headers: {
          "retry-after": String(rateLimit.retryAfterSeconds),
        },
      },
    );
  }

  let payload: LoginPayload = {};
  try {
    payload = (await request.json()) as LoginPayload;
  } catch {
    return NextResponse.json(
      { code: "BAD_REQUEST", message: "invalid login payload" },
      { status: 400 },
    );
  }

  const password = payload.password?.trim() ?? "";
  if (password !== configuredPassword) {
    registerLoginFailure(ADMIN_LOGIN_SCOPE, clientIp);
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "invalid password" },
      { status: 401 },
    );
  }

  clearLoginFailures(ADMIN_LOGIN_SCOPE, clientIp);

  let token: string;
  try {
    token = await issueSessionToken("admin", ADMIN_SESSION_TTL_SECONDS, sessionSecret);
  } catch {
    return NextResponse.json(
      { code: "INTERNAL_ERROR", message: "failed to issue session token" },
      { status: 500 },
    );
  }

  const cookieStore = await cookies();
  cookieStore.set(ADMIN_SESSION_COOKIE, token, {
    httpOnly: true,
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: ADMIN_SESSION_TTL_SECONDS,
  });

  return NextResponse.json({ ok: true });
}
