import { cookies } from "next/headers";
import { NextResponse } from "next/server";

import {
  SITE_SESSION_COOKIE,
  getPrivateSitePasswordFromEnv,
  isPrivateModeActiveFromEnv,
} from "@/lib/site-auth";

type LoginPayload = {
  password?: string;
};

export async function POST(request: Request) {
  if (!isPrivateModeActiveFromEnv()) {
    return NextResponse.json({ ok: true, disabled: true });
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
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "invalid password" },
      { status: 401 },
    );
  }

  const cookieStore = await cookies();
  cookieStore.set(SITE_SESSION_COOKIE, configuredPassword, {
    httpOnly: true,
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: 60 * 60 * 24 * 7,
  });

  return NextResponse.json({ ok: true });
}
