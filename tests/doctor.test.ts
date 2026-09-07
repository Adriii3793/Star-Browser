import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';

import { nativePackages } from '../scripts/doctor.mjs';

const lock = readFileSync(new URL('../package-lock.json', import.meta.url), 'utf8');
const packaged = new Set(
	[...lock.matchAll(/"node_modules\/(@[^"/]+\/[^"/]+)"/g)].map((match) => match[1])
);

const machines: [string, string, boolean][] = [
	['darwin', 'arm64', false],
	['darwin', 'x64', false],
	['win32', 'x64', false],
	['win32', 'arm64', false],
	['win32', 'ia32', false],
	['linux', 'x64', false],
	['linux', 'x64', true],
	['linux', 'arm64', false],
	['linux', 'arm64', true],
	['linux', 'arm', false],
	['linux', 'arm', true],
	['linux', 'riscv64', false]
];

describe('the native packages a machine needs', () => {
	for (const [os, cpu, musl] of machines) {
		it(`names published packages for ${os} ${cpu}${musl ? ' (musl)' : ''}`, () => {
			const wanted = nativePackages(os, cpu, musl);
			assert.ok(wanted, `no mapping for ${os}/${cpu}`);
			for (const [scope, name] of Object.entries(wanted)) {
				assert.ok(
					packaged.has(`${scope}/${name}`),
					`${scope}/${name} is not a package the lockfile knows about`
				);
			}
		});
	}

	it('picks the musl build only when asked', () => {
		assert.equal(nativePackages('linux', 'x64', false)['@tauri-apps'], 'cli-linux-x64-gnu');
		assert.equal(nativePackages('linux', 'x64', true)['@tauri-apps'], 'cli-linux-x64-musl');
	});

	it('handles the Linux targets that break the pattern', () => {
		assert.equal(nativePackages('linux', 'arm', false)['@tauri-apps'], 'cli-linux-arm-gnueabihf');
		assert.equal(nativePackages('linux', 'arm', true)['@rollup'], 'rollup-linux-arm-musleabihf');
		assert.equal(nativePackages('linux', 'riscv64', true)['@tauri-apps'], 'cli-linux-riscv64-gnu');
	});

	it('admits when a platform has no prebuilt binaries', () => {
		assert.equal(nativePackages('freebsd', 'x64', false), null);
		assert.equal(nativePackages('android', 'arm64', false), null);
	});
});
