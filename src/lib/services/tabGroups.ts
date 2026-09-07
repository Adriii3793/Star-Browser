export interface GroupedTab {
	id: string;
	groupId?: string;
}

function lastMemberOf(list: readonly GroupedTab[], groupId: string): number {
	let last = -1;
	for (let i = 0; i < list.length; i++) {
		if (list[i].groupId === groupId) last = i;
	}
	return last;
}

function hasOtherMember(list: readonly GroupedTab[], groupId: string, except: string): boolean {
	return list.some((tab) => tab.id !== except && tab.groupId === groupId);
}

export function withGroup<T extends GroupedTab>(
	list: readonly T[],
	tabId: string,
	groupId: string | undefined
): T[] {
	const from = list.findIndex((tab) => tab.id === tabId);
	if (from === -1) return list as T[];
	const tab = list[from];
	if ((tab.groupId ?? undefined) === (groupId ?? undefined)) return list as T[];

	const next = list.filter((_, i) => i !== from);
	let at = from;
	if (tab.groupId) {
		const last = lastMemberOf(next, tab.groupId);
		if (last !== -1) at = last + 1;
	}
	if (groupId) {
		const last = lastMemberOf(next, groupId);
		if (last !== -1) at = last + 1;
	}
	next.splice(at, 0, { ...tab, groupId } as T);
	return next;
}

export function reorder<T extends GroupedTab>(list: readonly T[], from: number, to: number): T[] {
	if (from === to || from < 0 || from >= list.length || to < 0 || to >= list.length) {
		return list as T[];
	}
	const next = list.filter((_, i) => i !== from);
	const tab = list[from];
	const before = next[to - 1]?.groupId;
	const after = next[to]?.groupId;

	let groupId: string | undefined;
	if (before && before === after) groupId = before;
	else if (tab.groupId && (before === tab.groupId || after === tab.groupId)) groupId = tab.groupId;
	else if (tab.groupId && !hasOtherMember(list, tab.groupId, tab.id)) groupId = tab.groupId;

	next.splice(to, 0, { ...tab, groupId } as T);
	return next;
}

export function dropFromGroup<T extends GroupedTab>(list: readonly T[], index: number): T[] {
	const tab = list[index];
	if (!tab?.groupId) return list as T[];
	if (list[index - 1]?.groupId === tab.groupId && list[index + 1]?.groupId === tab.groupId) {
		return list as T[];
	}
	const next = [...list];
	next[index] = { ...tab, groupId: undefined } as T;
	return next;
}

export function escapedGroup(list: readonly GroupedTab[], index: number): boolean {
	const tab = list[index];
	if (!tab?.groupId) return false;
	if (list[index - 1]?.groupId === tab.groupId) return false;
	if (list[index + 1]?.groupId === tab.groupId) return false;
	return hasOtherMember(list, tab.groupId, tab.id);
}

export function contiguousGroups<T extends GroupedTab>(list: readonly T[]): T[] {
	const placed = new Set<string>();
	const out: T[] = [];
	for (const tab of list) {
		if (!tab.groupId) {
			out.push(tab);
			continue;
		}
		if (placed.has(tab.groupId)) continue;
		placed.add(tab.groupId);
		for (const member of list) {
			if (member.groupId === tab.groupId) out.push(member);
		}
	}
	return out;
}

export function liveGroups<G extends { id: string }>(groups: readonly G[], list: readonly GroupedTab[]): G[] {
	const kept = groups.filter((group) => list.some((tab) => tab.groupId === group.id));
	return kept.length === groups.length ? (groups as G[]) : kept;
}
