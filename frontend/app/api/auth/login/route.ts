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
  SITE_SESSION_COOKIE,
  SITE_SESSION_TTL_SECONDS,
  getPrivateSitePasswordFromEnv,
  isPrivateModeActiveFromEnv,
} from "@/lib/site-auth";

type LoginPayload = {
  password?: string;
};

const SITE_LOGIN_SCOPE = "site_login";

export async function POST(request: Request) {
  if (!isPrivateModeActiveFromEnv()) {
    return NextResponse.json({ ok: true, disabled: true });
  }

  const sessionSecret = getSessionSecretFromEnv();
  if (!sessionSecret) {
    return NextResponse.json(
      { code: "SESSION_DISABLED", message: "session secret is not configured" },
      { status: 503 },
    );
  }

  const clientIp = resolveClientIpFromHeaders(request.headers);
  const rateLimit = checkLoginRateLimit(SITE_LOGIN_SCOPE, clientIp);
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

  const configuredPassword = getPrivateSitePasswordFromEnv();
  const password = payload.password?.trim() ?? "";

  if (password !== configuredPassword) {
    registerLoginFailure(SITE_LOGIN_SCOPE, clientIp);
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "invalid password" },
      { status: 401 },
    );
  }

  clearLoginFailures(SITE_LOGIN_SCOPE, clientIp);

  let token: string;
  try {
    token = await issueSessionToken("site", SITE_SESSION_TTL_SECONDS, sessionSecret);
  } catch {
    return NextResponse.json(
      { code: "INTERNAL_ERROR", message: "failed to issue session token" },
      { status: 500 },
    );
  }

  const cookieStore = await cookies();
  cookieStore.set(SITE_SESSION_COOKIE, token, {
    httpOnly: true,
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: SITE_SESSION_TTL_SECONDS,
  });

  return NextResponse.json({ ok: true });
}
