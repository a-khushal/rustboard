import type { Ellipse, Rectangle, Line, Arrow, Diamond, Text } from '$lib/stores/editor';

const DEFAULT_FONT_SIZE = 16;
export const DEFAULT_TEXT_FONT_SIZE = DEFAULT_FONT_SIZE;
export const TEXT_LINE_HEIGHT_RATIO = 1.25;
const DEFAULT_TEXT_ASCENT = 12;
const DEFAULT_TEXT_DESCENT = 4;
export const TEXT_HORIZONTAL_PADDING = 4;
export const TEXT_VERTICAL_PADDING = 4;

export function getFontForSize(fontSize: number = DEFAULT_FONT_SIZE): string {
	return `${fontSize}px 'Lucida Console', monospace`;
}

export function measureTextBounds(
	textValue: string,
	ctx?: CanvasRenderingContext2D,
	fontSize: number = DEFAULT_FONT_SIZE
): { width: number; ascent: number; descent: number; height: number } {
	const targetText = textValue ?? '';

	if (!ctx) {
		const scale = fontSize / DEFAULT_FONT_SIZE;
		const fallbackWidth = Math.max(targetText.length, 1) * 8 * scale;
		const ascent = DEFAULT_TEXT_ASCENT * scale;
		const descent = DEFAULT_TEXT_DESCENT * scale;
		return { width: fallbackWidth, ascent, descent, height: ascent + descent };
	}

	ctx.save();
	ctx.font = getFontForSize(fontSize);
	const metrics = ctx.measureText(targetText || ' ');
	ctx.restore();

	const ascent = metrics.actualBoundingBoxAscent || DEFAULT_TEXT_ASCENT * (fontSize / DEFAULT_FONT_SIZE);
	const descent = metrics.actualBoundingBoxDescent || DEFAULT_TEXT_DESCENT * (fontSize / DEFAULT_FONT_SIZE);
	return {
		width: metrics.width || Math.max(targetText.length, 1) * 8 * (fontSize / DEFAULT_FONT_SIZE),
		ascent,
		descent,
		height: ascent + descent,
	};
}

export function wrapTextToWidth(
	text: string,
	maxWidth: number,
	fontSize: number = DEFAULT_FONT_SIZE,
	ctx?: CanvasRenderingContext2D
): string[] {
	if (!ctx || maxWidth <= 0) {
		return text.split('\n');
	}

	ctx.save();
	ctx.font = getFontForSize(fontSize);

	const lines: string[] = [];
	const paragraphs = text.split('\n');

	for (const paragraph of paragraphs) {
		if (paragraph === '') {
			lines.push('');
			continue;
		}

		const words = paragraph.split(/\s+/);
		let currentLine = '';

		for (const word of words) {
			const testLine = currentLine ? `${currentLine} ${word}` : word;
			const metrics = ctx.measureText(testLine);

			if (metrics.width <= maxWidth) {
				currentLine = testLine;
			} else {
				if (currentLine) {
					lines.push(currentLine);
				}

				const wordMetrics = ctx.measureText(word);
				if (wordMetrics.width <= maxWidth) {
					currentLine = word;
				} else {
					let remainingWord = word;
					currentLine = '';
					while (remainingWord.length > 0) {
						let charIndex = 1;
						while (charIndex <= remainingWord.length) {
							const testChar = remainingWord.slice(0, charIndex);
							const charMetrics = ctx.measureText(testChar);
							if (charMetrics.width > maxWidth && charIndex > 1) {
								lines.push(remainingWord.slice(0, charIndex - 1));
								remainingWord = remainingWord.slice(charIndex - 1);
								break;
							}
							charIndex++;
						}
						if (charIndex > remainingWord.length) {
							if (remainingWord.length > 0) {
								lines.push(remainingWord);
							}
							remainingWord = '';
						}
					}
				}
			}
		}

		if (currentLine) {
			lines.push(currentLine);
		}
	}

	ctx.restore();
	return lines;
}

export function measureMultilineText(
	textValue: string,
	fontSize: number = DEFAULT_FONT_SIZE,
	ctx?: CanvasRenderingContext2D,
	maxWidth?: number
): {
	lines: string[];
	width: number;
	height: number;
	ascent: number;
	descent: number;
	lineHeight: number;
	lineWidths: number[];
} {
	let lines: string[];
	if (maxWidth !== undefined && maxWidth > 0 && ctx) {
		lines = wrapTextToWidth(textValue, maxWidth, fontSize, ctx);
	} else {
		const rawLines = textValue?.split('\n') ?? [''];
		lines = rawLines.length > 0 ? rawLines : [''];
	}

	const lineMetrics = lines.map(line => measureTextBounds(line || ' ', ctx, fontSize));
	const ascent = Math.max(...lineMetrics.map(m => m.ascent), DEFAULT_TEXT_ASCENT * (fontSize / DEFAULT_FONT_SIZE));
	const descent = Math.max(...lineMetrics.map(m => m.descent), DEFAULT_TEXT_DESCENT * (fontSize / DEFAULT_FONT_SIZE));
	const baseHeight = ascent + descent;
	const lineHeight = baseHeight * TEXT_LINE_HEIGHT_RATIO;
	const width = Math.max(...lineMetrics.map(m => Math.max(m.width, 1)), 1);
	const height = lines.length <= 1 ? baseHeight : baseHeight + (lines.length - 1) * lineHeight;

	return {
		lines,
		width,
		height,
		ascent,
		descent,
		lineHeight,
		lineWidths: lineMetrics.map(m => m.width),
	};
}

export function isPointInRectangle(x: number, y: number, rect: Rectangle): boolean {
	return (
		x >= rect.position.x &&
		x <= rect.position.x + rect.width &&
		y >= rect.position.y &&
		y <= rect.position.y + rect.height
	);
}

export function isPointInEllipse(x: number, y: number, ellipse: Ellipse): boolean {
	return (
		(x - ellipse.position.x) ** 2 / ellipse.radius_x ** 2 +
		(y - ellipse.position.y) ** 2 / ellipse.radius_y ** 2 <= 1
	);
}

export function isPointOnLine(x: number, y: number, line: Line | Arrow, threshold: number = 5): boolean {
	const { start, end } = line;
	const dx = end.x - start.x;
	const dy = end.y - start.y;
	const length = Math.sqrt(dx * dx + dy * dy);

	if (length === 0) return false;

	const t = Math.max(0, Math.min(1, ((x - start.x) * dx + (y - start.y) * dy) / (length * length)));
	const projX = start.x + t * dx;
	const projY = start.y + t * dy;
	const dist = Math.sqrt((x - projX) ** 2 + (y - projY) ** 2);

	return dist <= threshold;
}

export function isPointOnPath(x: number, y: number, path: { points: Array<{ x: number; y: number }> }, threshold: number = 5): boolean {
	if (path.points.length < 2) return false;
	
	for (let i = 0; i < path.points.length - 1; i++) {
		const start = path.points[i];
		const end = path.points[i + 1];
		const dx = end.x - start.x;
		const dy = end.y - start.y;
		const length = Math.sqrt(dx * dx + dy * dy);
		
		if (length === 0) continue;
		
		const t = Math.max(0, Math.min(1, ((x - start.x) * dx + (y - start.y) * dy) / (length * length)));
		const projX = start.x + t * dx;
		const projY = start.y + t * dy;
		const dist = Math.sqrt((x - projX) ** 2 + (y - projY) ** 2);
		
		if (dist <= threshold) return true;
	}
	
	return false;
}

export function rectangleIntersectsBox(rect: Rectangle, box: { x: number; y: number; width: number; height: number }): boolean {
	const rectBounds = { x: rect.position.x, y: rect.position.y, width: rect.width, height: rect.height };
	return (
		rectBounds.x >= box.x &&
		rectBounds.y >= box.y &&
		rectBounds.x + rectBounds.width <= box.x + box.width &&
		rectBounds.y + rectBounds.height <= box.y + box.height
	);
}

export function isPointInDiamond(x: number, y: number, diamond: Diamond): boolean {
	const centerX = diamond.position.x + diamond.width / 2;
	const centerY = diamond.position.y + diamond.height / 2;
	const halfWidth = diamond.width / 2;
	const halfHeight = diamond.height / 2;
	const dx = Math.abs(x - centerX);
	const dy = Math.abs(y - centerY);
	return (dx / halfWidth) + (dy / halfHeight) <= 1;
}

export function diamondIntersectsBox(diamond: Diamond, box: { x: number; y: number; width: number; height: number }): boolean {
	const centerX = diamond.position.x + diamond.width / 2;
	const centerY = diamond.position.y + diamond.height / 2;
	const halfWidth = diamond.width / 2;
	const halfHeight = diamond.height / 2;

	const corners = [
		{ x: centerX, y: centerY - halfHeight },
		{ x: centerX + halfWidth, y: centerY },
		{ x: centerX, y: centerY + halfHeight },
		{ x: centerX - halfWidth, y: centerY }
	];

	return corners.every(corner =>
		corner.x >= box.x && corner.x <= box.x + box.width &&
		corner.y >= box.y && corner.y <= box.y + box.height
	);
}

export function ellipseIntersectsBox(ellipse: Ellipse, box: { x: number; y: number; width: number; height: number }): boolean {
	const ellipseBounds = {
		x: ellipse.position.x - ellipse.radius_x,
		y: ellipse.position.y - ellipse.radius_y,
		width: ellipse.radius_x * 2,
		height: ellipse.radius_y * 2
	};

	if (
		ellipseBounds.x < box.x ||
		ellipseBounds.y < box.y ||
		ellipseBounds.x + ellipseBounds.width > box.x + box.width ||
		ellipseBounds.y + ellipseBounds.height > box.y + box.height
	) {
		return false;
	}

	for (let angle = 0; angle < Math.PI * 2; angle += Math.PI / 16) {
		const testX = ellipse.position.x + Math.cos(angle) * ellipse.radius_x;
		const testY = ellipse.position.y + Math.sin(angle) * ellipse.radius_y;

		if (
			testX < box.x || testX > box.x + box.width ||
			testY < box.y || testY > box.y + box.height
		) {
			return false;
		}
	}

	return true;
}

export function lineIntersectsBox(line: Line | Arrow, box: { x: number; y: number; width: number; height: number }): boolean {
	const lineStartInBox = (
		line.start.x >= box.x && line.start.x <= box.x + box.width &&
		line.start.y >= box.y && line.start.y <= box.y + box.height
	);
	const lineEndInBox = (
		line.end.x >= box.x && line.end.x <= box.x + box.width &&
		line.end.y >= box.y && line.end.y <= box.y + box.height
	);

	return lineStartInBox && lineEndInBox;
}

export function arrowIntersectsBox(arrow: Arrow, box: { x: number; y: number; width: number; height: number }): boolean {
	return lineIntersectsBox(arrow, box);
}

export function pathIntersectsBox(path: { points: Array<{ x: number; y: number }> }, box: { x: number; y: number; width: number; height: number }): boolean {
	if (path.points.length === 0) return false;
	
	const boxRight = box.x + box.width;
	const boxBottom = box.y + box.height;
	
	for (const point of path.points) {
		if (point.x >= box.x && point.x <= boxRight && point.y >= box.y && point.y <= boxBottom) {
			return true;
		}
	}
	
	for (let i = 0; i < path.points.length - 1; i++) {
		const start = path.points[i];
		const end = path.points[i + 1];
		const lineStartInBox = (
			start.x >= box.x && start.x <= boxRight &&
			start.y >= box.y && start.y <= boxBottom
		);
		const lineEndInBox = (
			end.x >= box.x && end.x <= boxRight &&
			end.y >= box.y && end.y <= boxBottom
		);
		if (lineStartInBox && lineEndInBox) {
			return true;
		}
	}
	
	return false;
}

export function isPointInText(x: number, y: number, text: Text, ctx?: CanvasRenderingContext2D): boolean {
	const contentWidth = text.boxWidth ? Math.max(10, text.boxWidth - TEXT_HORIZONTAL_PADDING * 2) : undefined;
	const layout = measureMultilineText(text.text, text.fontSize ?? DEFAULT_FONT_SIZE, ctx, contentWidth);
	const left = text.position.x - TEXT_HORIZONTAL_PADDING;
	const right = left + (text.boxWidth ?? (layout.width + TEXT_HORIZONTAL_PADDING * 2));
	const top = text.position.y - layout.ascent - TEXT_VERTICAL_PADDING;
	const bottom = top + layout.height + TEXT_VERTICAL_PADDING * 2;
	return (
		x >= left &&
		x <= right &&
		y >= top &&
		y <= bottom
	);
}

export function textIntersectsBox(text: Text, box: { x: number; y: number; width: number; height: number }, ctx?: CanvasRenderingContext2D): boolean {
	const contentWidth = text.boxWidth ? Math.max(10, text.boxWidth - TEXT_HORIZONTAL_PADDING * 2) : undefined;
	const layout = measureMultilineText(text.text, text.fontSize ?? DEFAULT_FONT_SIZE, ctx, contentWidth);
	const left = text.position.x - TEXT_HORIZONTAL_PADDING;
	const top = text.position.y - layout.ascent - TEXT_VERTICAL_PADDING;
	const textWidth = text.boxWidth ?? (layout.width + TEXT_HORIZONTAL_PADDING * 2);
	const textHeight = layout.height + TEXT_VERTICAL_PADDING * 2;
	return (
		left >= box.x &&
		top >= box.y &&
		left + textWidth <= box.x + box.width &&
		top + textHeight <= box.y + box.height
	);
}
