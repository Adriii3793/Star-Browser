import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
	faviconCandidates,
	faviconKey,
	faviconSources,
	faviconStep
} from '../src/lib/services/favicon.ts';

describe('where a site’s icon is looked for', () => {
	it('keeps the page’s own scheme and port', () => {
		assert.deepEqual(faviconSources('http://127.0.0.1:8777/declared-icon'), [
			'http://127.0.0.1:8777/favicon.ico',
			'http://127.0.0.1:8777/favicon.png',
			'http://127.0.0.1:8777/apple-touch-icon.png'
		]);
	});

	it('still resolves a bare hostname to https', () => {
		assert.equal(faviconSources('example.test')[0], 'https://example.test/favicon.ico');
	});

	it('has nothing to offer for a non-http address', () => {
		assert.deepEqual(faviconSources('data:text/html,hi'), []);
		assert.deepEqual(faviconSources(''), []);
	});

	it('puts what the page declared ahead of the guesses', () => {
		const candidates = faviconCandidates('https://astro.test/', 'https://astro.test/favicon.svg');
		assert.equal(candidates[0], 'https://astro.test/favicon.svg');
		assert.equal(candidates.length, 4);
	});
});

describe('which candidate a favicon shows', () => {
	const url = 'https://astro.test/';
	const declared = 'https://astro.test/favicon.svg';

	it('starts at the first candidate', () => {
		assert.equal(faviconStep(faviconCandidates(url, null), null), 0);
	});

	it('advances only while the list it recorded failures against is unchanged', () => {
		const guesses = faviconCandidates(url, null);
		const progress = { key: faviconKey(guesses), step: 2 };
		assert.equal(faviconStep(guesses, progress), 2);
	});

	it('tries an icon that arrives after every guess has already failed', () => {
		const guesses = faviconCandidates(url, null);
		const exhausted = { key: faviconKey(guesses), step: guesses.length };
		assert.equal(faviconStep(guesses, exhausted), guesses.length);

		const withDeclared = faviconCandidates(url, declared);
		assert.equal(faviconStep(withDeclared, exhausted), 0);
		assert.equal(withDeclared[faviconStep(withDeclared, exhausted)], declared);
	});

	it('restarts when the page changes', () => {
		const first = faviconCandidates('https://one.test/', null);
		const progress = { key: faviconKey(first), step: 1 };
		assert.equal(faviconStep(faviconCandidates('https://two.test/', null), progress), 0);
	});
});
