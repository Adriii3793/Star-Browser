import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
	DISMISS_GRACE_MS,
	REOPEN_GUARD_MS,
	isStaleDismissal,
	opensNativeDialog,
	shouldDismissOnBlur,
	shouldDismissOnPointer,
	suppressesReopen
} from '../src/lib/services/popup.ts';

const shown = {
	shown: true,
	visible: true,
	shownAt: 0,
	now: DISMISS_GRACE_MS + 1,
	activeElement: null
};

describe('dismissing a clipped surface on focus loss', () => {
	it('dismisses once the surface has been up long enough', () => {
		assert.equal(shouldDismissOnBlur(shown), true);
	});

	it('ignores the focus change that put it on screen', () => {
		assert.equal(shouldDismissOnBlur({ ...shown, now: DISMISS_GRACE_MS - 1 }), false);
	});

	it('ignores focus loss while it is showing nothing', () => {
		assert.equal(shouldDismissOnBlur({ ...shown, shown: false }), false);
	});

	it('ignores focus loss once the host has hidden the webview', () => {
		assert.equal(shouldDismissOnBlur({ ...shown, visible: false }), false);
	});

	it('stays open while a native file or colour dialog has the focus', () => {
		for (const type of ['file', 'color', 'FILE', 'Color']) {
			assert.equal(
				shouldDismissOnBlur({ ...shown, activeElement: { tagName: 'INPUT', type } }),
				false,
				`input[type=${type}] should hold the surface open`
			);
		}
	});

	it('still dismisses when an ordinary field has the focus', () => {
		for (const element of [
			{ tagName: 'INPUT', type: 'text' },
			{ tagName: 'INPUT' },
			{ tagName: 'TEXTAREA' },
			{ tagName: 'BUTTON' }
		]) {
			assert.equal(shouldDismissOnBlur({ ...shown, activeElement: element }), true);
		}
	});

	it('recognises a dialog trigger by element, not by tag alone', () => {
		assert.equal(opensNativeDialog({ tagName: 'input', type: 'file' }), true);
		assert.equal(opensNativeDialog({ tagName: 'DIV', type: 'file' }), false);
		assert.equal(opensNativeDialog(null), false);
		assert.equal(opensNativeDialog(undefined), false);
	});
});

describe('a dismissal that arrives after the next popup has been asked for', () => {
	it('is ignored when it belongs to an earlier showing', () => {
		assert.equal(isStaleDismissal(1, 2), true);
	});

	it('is honoured when it belongs to the current showing', () => {
		assert.equal(isStaleDismissal(2, 2), false);
	});

	it('is honoured when it carries no stamp at all', () => {
		assert.equal(isStaleDismissal(undefined, 7), false);
	});

	it('survives the whole open A -> blur A -> open B -> late close A sequence', () => {
		let currentSeq = 0;
		let open: 'A' | 'B' | null = null;

		const show = (which: 'A' | 'B') => {
			currentSeq += 1;
			open = which;
			return currentSeq;
		};
		const dismissal = (seq: number) => {
			if (isStaleDismissal(seq, currentSeq)) return;
			open = null;
		};

		const seqA = show('A');
		const seqB = show('B');
		dismissal(seqA);
		assert.equal(open, 'B', "A's late dismissal must not close B");

		dismissal(seqB);
		assert.equal(open, null);
	});

	it('lets a popup be reopened immediately after a stale dismissal was ignored', () => {
		let currentSeq = 0;
		const show = () => ++currentSeq;
		const first = show();
		const second = show();
		assert.equal(isStaleDismissal(first, currentSeq), true);
		assert.equal(isStaleDismissal(second, currentSeq), false);
	});
});

describe('the toolbar toggle after a focus-loss dismissal', () => {
	it('treats a click in the dismissal window as the tail of that dismissal', () => {
		assert.equal(suppressesReopen(1_000, 1_000 + REOPEN_GUARD_MS - 1), true);
	});

	it('opens again once the window has passed', () => {
		assert.equal(suppressesReopen(1_000, 1_000 + REOPEN_GUARD_MS), false);
		assert.equal(suppressesReopen(1_000, 1_000 + REOPEN_GUARD_MS + 500), false);
	});

	it('does nothing when no focus-loss dismissal has happened', () => {
		assert.equal(suppressesReopen(0, 5_000), false);
	});
});

describe('dismissing a surface by pressing its backdrop', () => {
	const backdrop = { shown: true, shownAt: 0, now: DISMISS_GRACE_MS + 1 };

	it('dismisses once the surface has been up long enough', () => {
		assert.equal(shouldDismissOnPointer(backdrop), true);
	});

	it('ignores the press that put it on screen', () => {
		assert.equal(shouldDismissOnPointer({ ...backdrop, now: DISMISS_GRACE_MS - 1 }), false);
	});

	it('ignores a press while it is showing nothing', () => {
		assert.equal(shouldDismissOnPointer({ ...backdrop, shown: false }), false);
	});

	it('takes the grace period from the caller when it is given one', () => {
		assert.equal(shouldDismissOnPointer({ ...backdrop, now: 50, graceMs: 40 }), true);
		assert.equal(shouldDismissOnPointer({ ...backdrop, now: 30, graceMs: 40 }), false);
	});
});
