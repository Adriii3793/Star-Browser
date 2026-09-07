import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
	contiguousGroups,
	dropFromGroup,
	escapedGroup,
	liveGroups,
	reorder,
	withGroup,
	type GroupedTab
} from '../src/lib/services/tabGroups.ts';

function strip(spec: string): GroupedTab[] {
	return spec.split(/\s+/).filter(Boolean).map((token) => {
		const [id, groupId] = token.split(':');
		return groupId ? { id, groupId } : { id };
	});
}

function show(list: readonly GroupedTab[]): string {
	return list.map((tab) => (tab.groupId ? `${tab.id}:${tab.groupId}` : tab.id)).join(' ');
}

function runsDrawn(list: readonly GroupedTab[]): string[] {
	const out: string[] = [];
	let current: string | undefined;
	let open = false;
	for (const tab of list) {
		if (!open || tab.groupId !== current || !tab.groupId) {
			out.push(tab.groupId ?? '-');
			current = tab.groupId;
			open = true;
		}
	}
	return out.filter((id) => id !== '-');
}

function contiguous(list: readonly GroupedTab[]): boolean {
	const drawn = runsDrawn(list);
	return new Set(drawn).size === drawn.length;
}

describe('taking a tab out of a group', () => {
	it('steps the tab out past the group rather than leaving a hole in it', () => {
		const list = strip('A:g1 B:g1 C:g1 D');
		assert.equal(show(withGroup(list, 'B', undefined)), 'A:g1 C:g1 B D');
		assert.ok(contiguous(withGroup(list, 'B', undefined)));
	});

	it('takes the first tab out to the far side of the group', () => {
		const list = strip('A:g1 B:g1 C:g1 D');
		assert.equal(show(withGroup(list, 'A', undefined)), 'B:g1 C:g1 A D');
		assert.ok(contiguous(withGroup(list, 'A', undefined)));
	});

	it('leaves the last tab where it already is', () => {
		const list = strip('A:g1 B:g1 C:g1 D');
		assert.equal(show(withGroup(list, 'C', undefined)), 'A:g1 B:g1 C D');
	});

	it('leaves the only tab of a group where it already is', () => {
		const list = strip('A B:g2 C');
		assert.equal(show(withGroup(list, 'B', undefined)), 'A B C');
	});

	it('keeps every other tab, once each, in the same order', () => {
		const list = strip('A:g1 B:g1 C:g1 D:g2 E');
		for (const id of ['A', 'B', 'C']) {
			const after = withGroup(list, id, undefined);
			assert.equal(after.length, list.length, `${id}: no tab appears or disappears`);
			assert.equal(new Set(after.map((t) => t.id)).size, list.length, `${id}: no duplicates`);
			assert.deepEqual(
				after.filter((t) => t.id !== id).map((t) => t.id),
				list.filter((t) => t.id !== id).map((t) => t.id),
				`${id}: the tabs left behind keep their order`
			);
			assert.ok(contiguous(after), `${id}: every group is still one run`);
		}
	});

	it('survives being run over and over without settling in between', () => {
		let list = strip('A:g1 B:g1 C:g1 D:g1');
		for (const id of ['B', 'A', 'D', 'C']) {
			list = withGroup(list, id, undefined);
			assert.ok(contiguous(list));
		}
		assert.equal(show(list), 'C D A B');
		assert.equal(new Set(list.map((t) => t.id)).size, 4, 'no tab is duplicated');
		assert.ok(list.every((t) => !t.groupId), 'the group is empty by the end');
	});
});

describe('putting a tab into a group', () => {
	it('lands it after the group it is joining', () => {
		const list = strip('A:g1 B:g1 C D');
		assert.equal(show(withGroup(list, 'D', 'g1')), 'A:g1 B:g1 D:g1 C');
	});

	it('moves a tab straight from one group to another', () => {
		const list = strip('A:g1 B:g1 C:g2 D:g2');
		const after = withGroup(list, 'A', 'g2');
		assert.equal(show(after), 'B:g1 C:g2 D:g2 A:g2');
		assert.ok(contiguous(after));
	});

	it('starts a new group under a tab without moving it', () => {
		const list = strip('A B C');
		assert.equal(show(withGroup(list, 'B', 'gNew')), 'A B:gNew C');
	});

	it('starting a new group on a grouped tab takes it out of the old one first', () => {
		const list = strip('A:g1 B:g1 C:g1');
		const after = withGroup(list, 'B', 'gNew');
		assert.equal(show(after), 'A:g1 C:g1 B:gNew');
		assert.ok(contiguous(after));
	});

	it('does nothing when the tab is already in that group, or is not there at all', () => {
		const list = strip('A:g1 B');
		assert.equal(withGroup(list, 'A', 'g1'), list);
		assert.equal(withGroup(list, 'missing', 'g1'), list);
		assert.equal(withGroup(list, 'B', undefined), list);
	});
});

describe('dragging a tab along the strip', () => {
	it('joins the group a tab is dropped inside', () => {
		const list = strip('A:g1 B:g1 C');
		assert.equal(show(reorder(list, 2, 1)), 'A:g1 C:g1 B:g1');
	});

	it('leaves the group when the tab is dragged clear of it', () => {
		const list = strip('A:g1 B:g1 C');
		assert.equal(show(reorder(list, 1, 2)), 'A:g1 C B');
	});

	it('keeps a group of one alive when its tab is nudged along', () => {
		const list = strip('A B X:gx D');
		assert.equal(show(reorder(list, 2, 3)), 'A B D X:gx');
		assert.equal(show(reorder(list, 2, 0)), 'X:gx A B D');
	});

	it('keeps every tab and never splits a group', () => {
		const list = strip('A:g1 B:g1 C D:g2 E:g2');
		for (let from = 0; from < list.length; from++) {
			for (let to = 0; to < list.length; to++) {
				const after = reorder(list, from, to);
				assert.equal(after.length, list.length, `${from}->${to}`);
				assert.equal(new Set(after.map((t) => t.id)).size, list.length, `${from}->${to}`);
				assert.ok(contiguous(after), `${from}->${to} splits a group: ${show(after)}`);
			}
		}
	});

	it('returns the same list for a move that goes nowhere or off the ends', () => {
		const list = strip('A B');
		assert.equal(reorder(list, 1, 1), list);
		assert.equal(reorder(list, 0, 5), list);
		assert.equal(reorder(list, -1, 0), list);
	});
});

describe('whether a drag has taken a tab out of its group', () => {
	it('says so once the tab has no group neighbour left', () => {
		assert.equal(escapedGroup(strip('A:g1 B:g1 C'), 0), false);
		assert.equal(escapedGroup(strip('A:g1 C B:g1'), 1), false, 'C is not in g1 at all');
		assert.equal(escapedGroup(strip('C A:g1 B:g1'), 0), false);
		assert.equal(escapedGroup(strip('A:g1 B:g1 C'), 2), false, 'an ungrouped tab has nothing to leave');
	});

	it('says so for a grouped tab stranded away from its group', () => {
		assert.equal(escapedGroup(strip('A:g1 C B:g1'), 2), true);
	});

	it('never says so for the only tab of a group', () => {
		assert.equal(escapedGroup(strip('A B X:gx D'), 2), false);
		assert.equal(escapedGroup(strip('X:gx'), 0), false);
	});
});

describe('the groups that survive a change', () => {
	it('drops the ones nothing belongs to any more', () => {
		const groups = [{ id: 'g1' }, { id: 'g2' }];
		assert.deepEqual(liveGroups(groups, strip('A:g1 B')), [{ id: 'g1' }]);
		assert.deepEqual(liveGroups(groups, strip('A B')), []);
	});
});

describe('the order a saved session comes back in', () => {
	it('leaves a session this build wrote exactly as it was', () => {
		const list = strip('A:g1 B:g1 C D:g2 E:g2 F');
		assert.equal(show(contiguousGroups(list)), 'A:g1 B:g1 C D:g2 E:g2 F');
	});

	it('puts a group an older build left in two pieces back together', () => {
		const list = strip('A:g1 B C:g1 D');
		const after = contiguousGroups(list);
		assert.equal(show(after), 'A:g1 C:g1 B D');
		assert.ok(contiguous(after));
		assert.equal(new Set(after.map((t) => t.id)).size, list.length, 'no tab is lost or repeated');
	});

	it('keeps a group where its first tab was', () => {
		assert.equal(show(contiguousGroups(strip('A B:g1 C D:g1'))), 'A B:g1 D:g1 C');
	});

	it('untangles two groups written interleaved', () => {
		const list = strip('A:g1 B:g2 C:g1 D:g2 E');
		const after = contiguousGroups(list);
		assert.equal(show(after), 'A:g1 C:g1 B:g2 D:g2 E');
		assert.ok(contiguous(after));
		assert.equal(after.length, list.length);
	});
});

describe('stepping out of a group while dragging', () => {
	it('leaves the group where the tab stands, so a space opens there', () => {
		const list = strip('A:g1 B:g1 C:g2 D:g2');
		assert.equal(show(dropFromGroup(list, 1)), 'A:g1 B C:g2 D:g2');
		assert.ok(contiguous(dropFromGroup(list, 1)));
	});

	it('does not send a tab pulled off the left across to the right', () => {
		const list = strip('A:g1 B:g1 C');
		assert.equal(show(dropFromGroup(list, 0)), 'A B:g1 C');
	});

	it('refuses to cut a group in two', () => {
		const list = strip('A:g1 B:g1 C:g1');
		assert.equal(dropFromGroup(list, 1), list, 'the middle of a run cannot step out');
	});

	it('has nothing to do for a tab in no group', () => {
		const list = strip('A B:g1');
		assert.equal(dropFromGroup(list, 0), list);
	});

	it('lets the only tab of a group out', () => {
		assert.equal(show(dropFromGroup(strip('A X:gx B'), 1)), 'A X B');
	});

	it('and once out, the tab can still be dragged into the next group', () => {
		const stepped = dropFromGroup(strip('A:g1 B:g1 C:g2 D:g2'), 1);
		assert.equal(show(reorder(stepped, 1, 2)), 'A:g1 C:g2 B:g2 D:g2');
	});
});
