import { cookies } from "next/headers";
import { NextResponse } from "next/server";

import { ADMIN_SESSION_COOKIE, getAdminPasswordFromEnv } from "@/lib/admin-auth";

type LoginPayload = {
  password?: string;
};

export async function POST(request: Request) {
  const configuredPassword = getAdminPasswordFromEnv();
  if (!configuredPassword) {
    return NextResponse.json(
      { code: "ADMIN_DISABLED", message: "admin dashboard password is not configured" },
      { status: 503 },
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
    return NextResponse.json(
      { code: "UNAUTHORIZED", message: "invalid password" },
      { status: 401 },
    );
  }

  const cookieStore = await cookies();
  cookieStore.set(ADMIN_SESSION_COOKIE, configuredPassword, {
    httpOnly: true,
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: 60 * 60 * 24,
  });

  return NextResponse.json({ ok: true });
}
