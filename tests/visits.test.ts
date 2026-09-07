import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { continuesNavigation, heldVisitAddress } from '../src/lib/services/visits.ts';

describe('the address a settling visit keeps', () => {
	it('follows the redirects of an ordinary navigation', () => {
		const held = { url: 'https://example.test/' };
		assert.equal(heldVisitAddress(held, 'https://www.example.test/home', null), 'https://www.example.test/home');
	});

	it('records the first address when nothing is held yet', () => {
		assert.equal(heldVisitAddress(undefined, 'https://example.test/', null), 'https://example.test/');
		assert.equal(heldVisitAddress(undefined, 'https://search.test/?q=cats', 'cats'), 'https://search.test/?q=cats');
	});

	it('keeps a search on the address it was issued at', () => {
		const issued = { url: 'https://search.test/search?q=cats' };
		assert.equal(
			heldVisitAddress(issued, 'https://search.test/search?q=cats&sei=Ab12Cd', 'cats'),
			'https://search.test/search?q=cats'
		);
	});

	it('holds that address across every hop of the search, not just the first', () => {
		const issued = { url: 'https://search.test/search?q=cats' };
		let address = heldVisitAddress(issued, 'https://search.test/search?q=cats&sei=one', 'cats');
		address = heldVisitAddress({ url: address }, 'https://search.test/search?q=cats&sei=two', 'cats');
		assert.equal(address, 'https://search.test/search?q=cats');
	});

	it('treats an empty query as a search, because an empty search is still a search', () => {
		const issued = { url: 'https://search.test/search?q=' };
		assert.equal(heldVisitAddress(issued, 'https://search.test/search?q=&sei=x', ''), 'https://search.test/search?q=');
	});
});

describe('whether a reported address continues the navigation under way', () => {
	const WINDOW = 1500;

	it('follows a navigation the shell started, however slow the server is', () => {
		assert.equal(continuesNavigation(true, 3000, WINDOW), true);
		assert.equal(continuesNavigation(true, 30_000, WINDOW), true);
	});

	it('still folds in a hop that follows the action closely', () => {
		assert.equal(continuesNavigation(false, 200, WINDOW), true);
	});

	it('starts a new entry once the tab has answered and the window has passed', () => {
		assert.equal(continuesNavigation(false, 3000, WINDOW), false);
	});
});
