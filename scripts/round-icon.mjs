#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { inflateSync, deflateSync, crc32 } from 'node:zlib';

const PNG_MAGIC = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

function readChunks(buf) {
	if (!buf.subarray(0, 8).equals(PNG_MAGIC)) throw new Error('not a PNG file');
	const chunks = [];
	let off = 8;
	while (off < buf.length) {
		const length = buf.readUInt32BE(off);
		const type = buf.toString('ascii', off + 4, off + 8);
		const data = buf.subarray(off + 8, off + 8 + length);
		chunks.push({ type, data });
		off += 12 + length;
	}
	return chunks;
}

function paeth(a, b, c) {
	const p = a + b - c;
	const pa = Math.abs(p - a);
	const pb = Math.abs(p - b);
	const pc = Math.abs(p - c);
	if (pa <= pb && pa <= pc) return a;
	return pb <= pc ? b : c;
}

function decode(buf) {
	const chunks = readChunks(buf);
	const ihdr = chunks.find((c) => c.type === 'IHDR');
	if (!ihdr) throw new Error('PNG has no IHDR chunk');

	const width = ihdr.data.readUInt32BE(0);
	const height = ihdr.data.readUInt32BE(4);
	const depth = ihdr.data[8];
	const colorType = ihdr.data[9];
	const interlace = ihdr.data[12];

	if (depth !== 8) throw new Error(`unsupported bit depth ${depth} (need 8)`);
	if (interlace !== 0) throw new Error('interlaced PNGs are not supported');
	if (colorType !== 6 && colorType !== 2) {
		throw new Error(`unsupported colour type ${colorType} (need 2=RGB or 6=RGBA)`);
	}

	const channels = colorType === 6 ? 4 : 3;
	const stride = width * channels;
	const idat = Buffer.concat(chunks.filter((c) => c.type === 'IDAT').map((c) => c.data));
	const raw = inflateSync(idat);

	const out = Buffer.alloc(height * stride);
	let pos = 0;
	for (let y = 0; y < height; y++) {
		const filter = raw[pos++];
		const line = raw.subarray(pos, pos + stride);
		pos += stride;
		const cur = out.subarray(y * stride, (y + 1) * stride);
		const prev = y > 0 ? out.subarray((y - 1) * stride, y * stride) : null;

		for (let x = 0; x < stride; x++) {
			const left = x >= channels ? cur[x - channels] : 0;
			const up = prev ? prev[x] : 0;
			const upLeft = prev && x >= channels ? prev[x - channels] : 0;
			let value = line[x];
			if (filter === 1) value += left;
			else if (filter === 2) value += up;
			else if (filter === 3) value += (left + up) >> 1;
			else if (filter === 4) value += paeth(left, up, upLeft);
			else if (filter !== 0) throw new Error(`unknown scanline filter ${filter}`);
			cur[x] = value & 0xff;
		}
	}

	if (channels === 4) return { width, height, pixels: out };
	const rgba = Buffer.alloc(width * height * 4, 0xff);
	for (let i = 0, j = 0; i < out.length; i += 3, j += 4) {
		rgba[j] = out[i];
		rgba[j + 1] = out[i + 1];
		rgba[j + 2] = out[i + 2];
	}
	return { width, height, pixels: rgba };
}

function chunk(type, data) {
	const head = Buffer.alloc(8);
	head.writeUInt32BE(data.length, 0);
	head.write(type, 4, 'ascii');
	const body = Buffer.concat([head.subarray(4, 8), data]);
	const tail = Buffer.alloc(4);
	tail.writeUInt32BE(crc32(body) >>> 0, 0);
	return Buffer.concat([head, data, tail]);
}

function encode({ width, height, pixels }) {
	const stride = width * 4;
	const raw = Buffer.alloc(height * (stride + 1));
	for (let y = 0; y < height; y++) {
		raw[y * (stride + 1)] = 0;
		pixels.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
	}

	const ihdr = Buffer.alloc(13);
	ihdr.writeUInt32BE(width, 0);
	ihdr.writeUInt32BE(height, 4);
	ihdr[8] = 8;
	ihdr[9] = 6;
	ihdr[10] = 0;
	ihdr[11] = 0;
	ihdr[12] = 0;

	return Buffer.concat([
		PNG_MAGIC,
		chunk('IHDR', ihdr),
		chunk('IDAT', deflateSync(raw, { level: 9 })),
		chunk('IEND', Buffer.alloc(0))
	]);
}

function coverage(px, py, w, h, r) {
	const STEPS = 4;
	let inside = 0;
	for (let sy = 0; sy < STEPS; sy++) {
		for (let sx = 0; sx < STEPS; sx++) {
			const x = px + (sx + 0.5) / STEPS;
			const y = py + (sy + 0.5) / STEPS;
			const dx = x < r ? r - x : x > w - r ? x - (w - r) : 0;
			const dy = y < r ? r - y : y > h - r ? y - (h - r) : 0;
			if (dx * dx + dy * dy <= r * r) inside++;
		}
	}
	return inside / (STEPS * STEPS);
}

function roundCorners(image, ratio) {
	const { width, height, pixels } = image;
	const radius = Math.min(width, height) * ratio;
	for (let y = 0; y < height; y++) {
		for (let x = 0; x < width; x++) {
			const c = coverage(x, y, width, height, radius);
			if (c >= 1) continue;
			const i = (y * width + x) * 4 + 3;
			pixels[i] = Math.round(pixels[i] * c);
		}
	}
	return image;
}

const args = process.argv.slice(2);
const flag = (name, fallback) => {
	const i = args.indexOf(`--${name}`);
	return i === -1 ? fallback : args[i + 1];
};
const positional = args.filter((a, i) => !a.startsWith('--') && !args[i - 1]?.startsWith('--'));

const source = positional[0] ?? 'src-tauri/app-icon.png';
const ratio = Number(flag('radius', '0.22'));
const out = flag('out', source);

if (!existsSync(source)) {
	console.error(`round-icon: no such file: ${source}`);
	process.exit(1);
}
if (!(ratio > 0 && ratio <= 0.5)) {
	console.error(`round-icon: --radius must be between 0 and 0.5, got ${ratio}`);
	process.exit(1);
}

const image = decode(readFileSync(source));
if (image.width !== image.height) {
	console.warn(`round-icon: ${source} is ${image.width}x${image.height}, not square — corners may look uneven`);
}

const alphaAt = (x, y) => image.pixels[(y * image.width + x) * 4 + 3];
const corners = [
	[0, 0],
	[image.width - 1, 0],
	[0, image.height - 1],
	[image.width - 1, image.height - 1]
];
if (corners.every(([x, y]) => alphaAt(x, y) === 0) && !args.includes('--force')) {
	console.log(`round-icon: ${source} already has rounded corners — nothing to do (pass --force to round it again).`);
	process.exit(0);
}

writeFileSync(out, encode(roundCorners(image, ratio)));
console.log(
	`round-icon: wrote ${out} (${image.width}x${image.height}, radius ${Math.round(ratio * 100)}%)`
);
console.log('round-icon: now run `npm run icons` to regenerate the bundled icon set.');
