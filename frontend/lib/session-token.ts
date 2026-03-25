export type SessionTokenKind = "site" | "admin";

const TOKEN_VERSION = "v1";

export function getSessionSecretFromEnv(): string {
  return process.env.SESSION_SECRET?.trim() ?? "";
}

export function isSessionSecretConfigured(): boolean {
  return getSessionSecretFromEnv().length > 0;
}

export async function issueSessionToken(
  kind: SessionTokenKind,
  ttlSeconds: number,
  secret: string,
): Promise<string> {
  if (!secret) {
    throw new Error("session secret is empty");
  }

  const now = nowSeconds();
  const exp = now + Math.max(60, Math.floor(ttlSeconds));
  const nonce = randomHex(16);
  const message = `${TOKEN_VERSION}.${kind}.${exp}.${nonce}`;
  const signature = await signHmacHex(secret, message);
  return `${message}.${signature}`;
}

export async function verifySessionToken(
  token: string,
  expectedKind: SessionTokenKind,
  secret: string,
): Promise<boolean> {
  if (!secret) {
    return false;
  }

  const parts = token.split(".");
  if (parts.length !== 5) {
    return false;
  }

  const [version, kind, expRaw, nonce, signature] = parts;
  if (version !== TOKEN_VERSION || kind !== expectedKind || !nonce) {
    return false;
  }

  const exp = Number(expRaw);
  if (!Number.isFinite(exp) || exp <= nowSeconds()) {
    return false;
  }

  const message = `${version}.${kind}.${expRaw}.${nonce}`;
  const expectedSignature = await signHmacHex(secret, message);
  return timingSafeEqual(signature, expectedSignature);
}

async function signHmacHex(secret: string, message: string): Promise<string> {
  const keyData = new TextEncoder().encode(secret);
  const messageData = new TextEncoder().encode(message);

  const key = await crypto.subtle.importKey(
    "raw",
    keyData,
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );

  const signature = await crypto.subtle.sign("HMAC", key, messageData);
  return toHex(new Uint8Array(signature));
}

function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}

function randomHex(bytes: number): string {
  const arr = new Uint8Array(bytes);
  crypto.getRandomValues(arr);
  return toHex(arr);
}

function toHex(bytes: Uint8Array): string {
  let out = "";
  for (const byte of bytes) {
    out += byte.toString(16).padStart(2, "0");
  }
  return out;
}

function timingSafeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) {
    return false;
  }

  let diff = 0;
  for (let i = 0; i < a.length; i += 1) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}
