const RATE_LIMIT_WINDOW_MS = 5 * 60 * 1000;
const RATE_LIMIT_MAX_ATTEMPTS = 10;

type Bucket = {
  attempts: number[];
};

const buckets = new Map<string, Bucket>();

export function resolveClientIpFromHeaders(headers: Headers): string {
  const forwarded = headers.get("x-forwarded-for");
  if (forwarded) {
    const first = forwarded.split(",")[0]?.trim();
    if (first) {
      return first;
    }
  }

  const realIp = headers.get("x-real-ip")?.trim();
  if (realIp) {
    return realIp;
  }

  return "unknown";
}

export function checkLoginRateLimit(scope: string, ip: string): {
  limited: boolean;
  retryAfterSeconds: number;
} {
  const key = `${scope}:${ip}`;
  const now = Date.now();
  const bucket = ensureBucket(key);
  prune(bucket, now);

  if (bucket.attempts.length < RATE_LIMIT_MAX_ATTEMPTS) {
    return { limited: false, retryAfterSeconds: 0 };
  }

  const earliest = bucket.attempts[0] ?? now;
  const retryAfterMs = Math.max(0, earliest + RATE_LIMIT_WINDOW_MS - now);
  const retryAfterSeconds = Math.max(1, Math.ceil(retryAfterMs / 1000));
  return { limited: true, retryAfterSeconds };
}

export function registerLoginFailure(scope: string, ip: string) {
  const key = `${scope}:${ip}`;
  const now = Date.now();
  const bucket = ensureBucket(key);
  prune(bucket, now);
  bucket.attempts.push(now);
}

export function clearLoginFailures(scope: string, ip: string) {
  const key = `${scope}:${ip}`;
  buckets.delete(key);
}

function ensureBucket(key: string): Bucket {
  const bucket = buckets.get(key);
  if (bucket) {
    return bucket;
  }

  const created: Bucket = { attempts: [] };
  buckets.set(key, created);
  return created;
}

function prune(bucket: Bucket, now: number) {
  const threshold = now - RATE_LIMIT_WINDOW_MS;
  bucket.attempts = bucket.attempts.filter((ts) => ts > threshold);
}
