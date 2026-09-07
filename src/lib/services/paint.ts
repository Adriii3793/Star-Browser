function alphaOf(color: string): number {
	if (!color || color === 'transparent') return 0;
	const call = color.match(/^([a-z-]+)\(([^)]*)\)$/i);
	if (!call) return 1;

	const body = call[2];
	const slash = body.lastIndexOf('/');
	if (slash >= 0) {
		const alpha = Number.parseFloat(body.slice(slash + 1));
		return Number.isFinite(alpha) ? alpha : 1;
	}

	const name = call[1].toLowerCase();
	if (name !== 'rgba' && name !== 'hsla' && name !== 'rgb' && name !== 'hsl') return 1;
	const parts = body.split(/[\s,]+/).filter(Boolean);
	if (parts.length !== 4) return 1;
	const alpha = Number.parseFloat(parts[3]);
	return Number.isFinite(alpha) ? alpha : 1;
}

export interface PaintStyle {
	backgroundColor: string;
	backgroundImage: string;
	borderTopWidth: string;
	borderRightWidth: string;
	borderBottomWidth: string;
	borderLeftWidth: string;
	borderTopColor: string;
	borderRightColor: string;
	borderBottomColor: string;
	borderLeftColor: string;
}

export function paintsAnything(style: PaintStyle): boolean {
	if (style.backgroundImage && style.backgroundImage !== 'none') return true;
	if (alphaOf(style.backgroundColor) > 0) return true;

	return (
		[
			[style.borderTopWidth, style.borderTopColor],
			[style.borderRightWidth, style.borderRightColor],
			[style.borderBottomWidth, style.borderBottomColor],
			[style.borderLeftWidth, style.borderLeftColor]
		] as const
	).some(([width, color]) => (Number.parseFloat(width) || 0) > 0 && alphaOf(color) > 0);
}
