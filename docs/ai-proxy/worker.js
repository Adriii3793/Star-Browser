const OPENROUTER = 'https://openrouter.ai/api/v1/chat/completions';

const ALLOWED_MODELS = new Set([
	'nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free',
	'google/gemma-4-26b-a4b-it:free'
]);

const MAX_BODY_BYTES = 128 * 1024;
const RATE_LIMIT = 40;
const WINDOW_SECONDS = 3600;

function json(status, body) {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'content-type': 'application/json' }
	});
}

async function overLimit(env, ip) {
	if (!env.RATE) return false;
	const key = `rl:${ip}`;
	const used = Number((await env.RATE.get(key)) ?? 0);
	if (used >= RATE_LIMIT) return true;
	await env.RATE.put(key, String(used + 1), { expirationTtl: WINDOW_SECONDS });
	return false;
}

export default {
	async fetch(request, env) {
		if (request.method !== 'POST') {
			return json(405, { error: { message: 'POST only' } });
		}
		if (!env.OPENROUTER_API_KEY) {
			return json(500, { error: { message: 'Proxy is missing OPENROUTER_API_KEY' } });
		}

		const ip = request.headers.get('cf-connecting-ip') ?? 'unknown';
		if (await overLimit(env, ip)) {
			return json(429, { error: { message: 'Rate limit reached. Try again later.' } });
		}

		const raw = await request.text();
		if (raw.length > MAX_BODY_BYTES) {
			return json(413, { error: { message: 'Request too large' } });
		}

		let body;
		try {
			body = JSON.parse(raw);
		} catch {
			return json(400, { error: { message: 'Body must be JSON' } });
		}

		if (!ALLOWED_MODELS.has(body?.model)) {
			return json(400, { error: { message: 'Unsupported model' } });
		}
		if (!Array.isArray(body?.messages) || body.messages.length === 0) {
			return json(400, { error: { message: 'messages must be a non-empty array' } });
		}

		const upstream = await fetch(OPENROUTER, {
			method: 'POST',
			headers: {
				authorization: `Bearer ${env.OPENROUTER_API_KEY}`,
				'content-type': 'application/json'
			},
			body: JSON.stringify({ model: body.model, messages: body.messages })
		});

		return new Response(await upstream.text(), {
			status: upstream.status,
			headers: { 'content-type': 'application/json' }
		});
	}
};
