import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const OS_LABEL = { darwin: 'macOS', win32: 'Windows', linux: 'Linux' };

function usesMusl() {
	try {
		return !process.report.getReport().header.glibcVersionRuntime;
	} catch {
		return false;
	}
}

export function nativePackages(os, cpu, musl = os === 'linux' && usesMusl()) {
	const libc = musl ? 'musl' : 'gnu';
	let tauri = null;
	let rollup = null;
	if (os === 'darwin') {
		tauri = `cli-darwin-${cpu}`;
		rollup = `rollup-darwin-${cpu}`;
	} else if (os === 'win32') {
		tauri = `cli-win32-${cpu}-msvc`;
		rollup = `rollup-win32-${cpu}-msvc`;
	} else if (os === 'linux') {
		if (cpu === 'arm') {
			tauri = 'cli-linux-arm-gnueabihf';
			rollup = `rollup-linux-arm-${musl ? 'musleabihf' : 'gnueabihf'}`;
		} else {
			tauri = cpu === 'riscv64' ? 'cli-linux-riscv64-gnu' : `cli-linux-${cpu}-${libc}`;
			rollup = `rollup-linux-${cpu}-${libc}`;
		}
	}
	if (!tauri || !rollup) return null;
	return {
		'@tauri-apps': tauri,
		'@esbuild': `${os}-${cpu}`,
		'@rollup': rollup
	};
}

function installed(scope) {
	try {
		return readdirSync(join(root, 'node_modules', scope), { withFileTypes: true })
			.filter((entry) => entry.isDirectory() || entry.isSymbolicLink())
			.map((entry) => entry.name);
	} catch {
		return [];
	}
}

function platformsAmong(names) {
	const found = new Set();
	for (const name of names) {
		for (const os of Object.keys(OS_LABEL)) if (name.includes(os)) found.add(os);
	}
	return [...found];
}

function run(command, args) {
	try {
		return execFileSync(command, args, {
			encoding: 'utf8',
			stdio: ['ignore', 'pipe', 'ignore']
		}).trim();
	} catch {
		return null;
	}
}

function report() {
	const { platform, arch } = process;
	const checks = [];
	const pass = (label, detail) => checks.push({ level: 'pass', label, detail });
	const warn = (label, detail) => checks.push({ level: 'warn', label, detail });
	const fail = (label, detail) => checks.push({ level: 'fail', label, detail });

	const nodeMajor = Number(process.versions.node.split('.')[0]);
	if (nodeMajor >= 18) pass('Node.js', `v${process.versions.node}`);
	else fail('Node.js', `v${process.versions.node} — Star needs 18 or newer, get it from nodejs.org`);

	const notes = [];
	if (/\s/.test(root)) notes.push('the path contains a space, which parts of the Rust toolchain mishandle');
	if (platform === 'darwin' && root.startsWith(join(homedir(), 'Downloads')))
		notes.push('it sits under ~/Downloads, where macOS restricts what may run');
	if (notes.length) warn('project path', [root, ...notes.map((n) => `      ${n}`)].join('\n'));
	else pass('project path', root);

	const wanted = nativePackages(platform, arch);
	if (!existsSync(join(root, 'node_modules'))) {
		fail('dependencies', 'node_modules is missing — run npm install');
	} else if (!wanted) {
		warn('dependencies', `no prebuilt binaries are published for ${platform}/${arch}`);
	} else {
		const missing = Object.entries(wanted).filter(([scope, name]) => !installed(scope).includes(name));
		if (missing.length === 0) {
			pass('dependencies', `native binaries present for ${OS_LABEL[platform] ?? platform} ${arch}`);
		} else {
			const strangers = platformsAmong(
				installed('@esbuild').concat(installed('@tauri-apps'))
			).filter((os) => os !== platform);
			const origin = strangers.length
				? `it was installed on ${strangers.map((os) => OS_LABEL[os]).join(' / ')}`
				: 'it is incomplete';
			fail(
				'dependencies',
				`node_modules cannot run here — ${origin}.\n      missing: ${missing
					.map(([scope, name]) => `${scope}/${name}`)
					.join(', ')}`
			);
		}
	}

	if (platform !== 'win32' && existsSync(join(root, 'node_modules/.bin/tauri.cmd')))
		warn(
			'dependencies',
			'node_modules/.bin holds Windows launcher scripts (.cmd), so this tree was built on Windows'
		);

	if (platform === 'darwin') {
		const quarantined = [root, join(root, 'package.json'), join(root, 'node_modules/.bin/tauri')].filter(
			(path) => existsSync(path) && run('xattr', ['-p', 'com.apple.quarantine', path]) !== null
		);
		if (quarantined.length)
			fail('macOS quarantine', 'files carry com.apple.quarantine, so macOS refuses to execute them');
		else pass('macOS quarantine', 'clear');
	}

	const rustc = run('rustc', ['--version']);
	if (rustc) pass('Rust', rustc);
	else fail('Rust', 'not installed — get it from https://rustup.rs');

	if (platform === 'darwin') {
		const clt = run('xcode-select', ['-p']);
		if (clt) pass('Xcode tools', clt);
		else fail('Xcode tools', 'missing — run xcode-select --install');
	} else if (platform === 'linux') {
		if (run('pkg-config', ['--exists', 'webkit2gtk-4.1']) !== null) pass('webkit2gtk', '4.1 found');
		else warn('webkit2gtk', '4.1 not found — install the -dev/-devel package for your distribution');
	} else if (platform === 'win32') {
		warn(
			'WebView2',
			'cannot be checked from here — bundled with Windows 11, otherwise install it from Microsoft'
		);
	}

	const envPath = join(root, '.env');
	if (!existsSync(envPath)) {
		warn('.env', 'missing — the AI features need OPENROUTER_API_KEY, see .env.example');
	} else {
		let contents = '';
		try {
			contents = readFileSync(envPath, 'utf8');
		} catch {
		}
		if (/^\s*OPENROUTER_API_KEY\s*=\s*\S/m.test(contents)) pass('.env', 'OPENROUTER_API_KEY is set');
		else warn('.env', 'OPENROUTER_API_KEY is empty or absent — the AI features stay off');
	}

	return checks;
}

function remedies(failed) {
	const hit = (label) => failed.some((check) => check.label === label);
	const steps = [];
	if (hit('Node.js')) steps.push('install Node.js 18+ from https://nodejs.org, then reopen the terminal');
	if (hit('Xcode tools')) steps.push('xcode-select --install');
	if (hit('Rust'))
		steps.push("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # then reopen the terminal");
	if (hit('dependencies')) steps.push('rm -rf node_modules src-tauri/target');
	if (hit('macOS quarantine')) steps.push('xattr -cr .');
	if (hit('dependencies'))
		steps.push('npm install   # leave package-lock.json alone, it already lists every platform');
	return steps;
}

function main() {
	const checks = report();
	const color = process.stdout.isTTY && !process.env.NO_COLOR;
	const paint = (code, text) => (color ? `\x1b[${code}m${text}\x1b[0m` : text);
	const MARK = { pass: paint('32', '  ok '), warn: paint('33', ' warn'), fail: paint('31', ' fail') };

	console.log('\nStar Browser — setup check');
	console.log(
		paint(
			'2',
			`${OS_LABEL[process.platform] ?? process.platform} ${process.arch} · node ${process.versions.node}\n`
		)
	);
	for (const { level, label, detail } of checks)
		console.log(`${MARK[level]}  ${label.padEnd(17)} ${detail}`);

	const failed = checks.filter((check) => check.level === 'fail');
	if (failed.length === 0) {
		console.log(`\n${paint('32', 'Ready.')} Start it with: npm run tauri dev\n`);
		return 0;
	}

	console.log(`\n${paint('1', 'To fix')} — from ${root}\n`);
	for (const step of remedies(failed)) console.log(`  ${step}`);
	console.log('\nThen run npm run doctor again.\n');
	return 1;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) process.exit(main());
