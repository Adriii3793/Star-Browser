import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';

const configUrl = new URL('../src-tauri/tauri.conf.json', import.meta.url);
const macosUrl = new URL('../src-tauri/tauri.macos.conf.json', import.meta.url);

const base = JSON.parse(readFileSync(configUrl, 'utf8'));
const macosPatch = JSON.parse(readFileSync(macosUrl, 'utf8'));

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

function mergePatch(doc: Json, patch: Json): Json {
	if (patch === null || typeof patch !== 'object' || Array.isArray(patch)) {
		return structuredClone(patch);
	}
	const target: { [key: string]: Json } =
		doc !== null && typeof doc === 'object' && !Array.isArray(doc) ? { ...doc } : {};
	for (const [key, value] of Object.entries(patch)) {
		if (value === null) delete target[key];
		else target[key] = mergePatch(target[key] ?? null, value);
	}
	return target;
}

const merged = mergePatch(base, macosPatch) as {
	app: { windows: Record<string, Json>[] };
};
const baseWindows = base.app.windows as Record<string, Json>[];
const macosWindows = merged.app.windows;

const MACOS_DIFFERENCES: Record<string, unknown> = { shadow: true };

describe('the macOS window configuration', () => {
	it('describes the same windows as the base configuration', () => {
		assert.equal(
			macosWindows.length,
			baseWindows.length,
			'the macOS patch replaces the whole `windows` array, so it has to list every window'
		);
	});

	for (const [index, baseWindow] of baseWindows.entries()) {
		const macosWindow = macosWindows[index] ?? {};
		const label = baseWindow.label ?? 'main';

		it(`window "${label}" leaves no key of the base entry undefined`, () => {
			const missing = Object.keys(baseWindow).filter((key) => !(key in macosWindow));
			assert.deepEqual(
				missing,
				[],
				`${missing.join(', ')} would silently fall back to Tauri's defaults on macOS`
			);
		});

		it(`window "${label}" differs from the base entry only where macOS needs it`, () => {
			for (const [key, value] of Object.entries(macosWindow)) {
				if (key in MACOS_DIFFERENCES) {
					assert.deepEqual(
						value,
						MACOS_DIFFERENCES[key],
						`\`${key}\` is a declared macOS difference and no longer holds its declared value`
					);
					continue;
				}
				if (!(key in baseWindow)) continue;
				assert.deepEqual(value, baseWindow[key], `\`${key}\` drifted from the base window entry`);
			}
		});

		it(`window "${label}" starts hidden on every platform`, () => {
			assert.equal(baseWindow.visible, false, 'base window must be created hidden');
			assert.equal(macosWindow.visible, false, 'macOS window must be created hidden');
		});
	}

	it('spells titleBarStyle the way Tauri parses it', () => {
		const accepted = ['Visible', 'Transparent', 'Overlay'];
		for (const [source, windows] of [
			['tauri.conf.json', baseWindows],
			['tauri.macos.conf.json', macosWindows]
		] as const) {
			for (const window of windows) {
				if (!('titleBarStyle' in window)) continue;
				assert.ok(
					accepted.includes(window.titleBarStyle as string),
					`${source}: "${window.titleBarStyle}" is not a TitleBarStyle — Tauri would read it as Visible`
				);
			}
		}
	});
});
