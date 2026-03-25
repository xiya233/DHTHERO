import { cookies } from "next/headers";
import { NextResponse } from "next/server";

import { SITE_SESSION_COOKIE } from "@/lib/site-auth";

export async function POST() {
  const cookieStore = await cookies();
  cookieStore.delete(SITE_SESSION_COOKIE);
  return NextResponse.json({ ok: true });
}
