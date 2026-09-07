import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { paintsAnything } from '../src/lib/services/paint.ts';

function style(over: Partial<Parameters<typeof paintsAnything>[0]> = {}) {
	return {
		backgroundColor: 'rgba(0, 0, 0, 0)',
		backgroundImage: 'none',
		borderTopWidth: '0px',
		borderRightWidth: '0px',
		borderBottomWidth: '0px',
		borderLeftWidth: '0px',
		borderTopColor: 'rgb(0, 0, 0)',
		borderRightColor: 'rgb(0, 0, 0)',
		borderBottomColor: 'rgb(0, 0, 0)',
		borderLeftColor: 'rgb(0, 0, 0)',
		...over
	};
}

function border(width: string, color = 'rgb(74, 58, 46)') {
	return {
		borderTopWidth: width,
		borderRightWidth: width,
		borderBottomWidth: width,
		borderLeftWidth: width,
		borderTopColor: color,
		borderRightColor: color,
		borderBottomColor: color,
		borderLeftColor: color
	};
}

describe('whether an element paints anything of its own', () => {
	it('says no for the menu anchor, which only positions the card', () => {
		assert.equal(paintsAnything(style()), false);
	});

	it('says yes for the menu card', () => {
		assert.equal(
			paintsAnything(style({ backgroundColor: 'rgb(255, 255, 255)', ...border('1px') })),
			true
		);
	});

	it('reads every spelling of a see-through background as no paint', () => {
		for (const color of [
			'transparent',
			'rgba(0, 0, 0, 0)',
			'rgba(74, 58, 46, 0)',
			'rgb(0 0 0 / 0)',
			''
		]) {
			assert.equal(paintsAnything(style({ backgroundColor: color })), false, color);
		}
	});

	it('finds the alpha in a modern colour function too', () => {
		for (const color of ['oklch(0 0 0 / 0)', 'color(srgb 0 0 0 / 0)', 'lab(0 0 0 / 0)']) {
			assert.equal(paintsAnything(style({ backgroundColor: color })), false, color);
		}
		for (const color of ['oklch(0.2 0 0)', 'color(srgb 1 0 0)', 'color(display-p3 1 0 0 / 0.5)']) {
			assert.equal(paintsAnything(style({ backgroundColor: color })), true, color);
		}
	});

	it('counts a barely-there background as paint', () => {
		assert.equal(paintsAnything(style({ backgroundColor: 'rgba(74, 58, 46, 0.28)' })), true);
		assert.equal(paintsAnything(style({ backgroundColor: 'rgba(0, 0, 0, 0.01)' })), true);
	});

	it('counts a background image as paint, whatever the colour under it', () => {
		assert.equal(paintsAnything(style({ backgroundImage: 'url("bg.png")' })), true);
		assert.equal(
			paintsAnything(style({ backgroundImage: 'linear-gradient(rgb(0, 0, 0), rgb(1, 1, 1))' })),
			true
		);
	});

	it('counts a border on any one side as paint', () => {
		const sides = ['Top', 'Right', 'Bottom', 'Left'] as const;
		for (const side of sides) {
			const one = { [`border${side}Width`]: '1px', [`border${side}Color`]: 'rgb(74, 58, 46)' };
			assert.equal(paintsAnything(style(one)), true, `border-${side.toLowerCase()} alone`);
		}
	});

	it('does not count a declared border that computes to no width', () => {
		assert.equal(paintsAnything(style(border('0px', 'rgb(74, 58, 46)'))), false);
	});

	it('does not count a full-width border that is transparent', () => {
		assert.equal(paintsAnything(style(border('1px', 'transparent'))), false);
		assert.equal(paintsAnything(style(border('2px', 'rgba(0, 0, 0, 0)'))), false);
	});
});
