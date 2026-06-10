export async function runAuthFlow(payload: Record<string, unknown>): Promise<unknown> {
  const provider = payload.provider as string;
  if (provider === 'stremio') {
    return { authUrl: 'https://www.stremio.com/login', mode: payload.mode };
  }
  if (provider === 'trakt') {
    const clientId = payload.clientId as string | undefined;
    if (clientId) {
      return { authUrl: `https://trakt.tv/oauth/authorize?client_id=${clientId}&response_type=code`, mode: payload.mode };
    }
  }
  return { error: `Unknown provider: ${provider}` };
}

export async function exchangeAuthCode(payload: Record<string, unknown>): Promise<unknown> {
  const url = payload.url as string | undefined;
  const body = payload.body as Record<string, string> | undefined;
  if (!url || !body) return null;
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  return res.ok ? res.json() : null;
}

export async function refreshAuthToken(payload: Record<string, unknown>): Promise<unknown> {
  return exchangeAuthCode(payload);
}
