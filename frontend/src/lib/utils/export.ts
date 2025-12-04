import type { Rectangle, Ellipse, Diamond, Line, Arrow, Text, Path } from '$lib/stores/editor';
import { TEXT_HORIZONTAL_PADDING, TEXT_VERTICAL_PADDING } from './geometry';
import { measureMultilineText, getTextContentWidthFromBoxWidth, getPathBoundingBox } from './geometry';

export async function exportToPNG(
	canvas: HTMLCanvasElement,
	filename: string = 'rustboard.png'
): Promise<void> {
	const dataUrl = canvas.toDataURL('image/png');
	const link = document.createElement('a');
	link.download = filename;
	link.href = dataUrl;
	link.click();
}

export function exportToSVG(
	rectangles: Rectangle[],
	ellipses: Ellipse[],
	diamonds: Diamond[],
	lines: Line[],
	arrows: Arrow[],
	texts: Text[],
	paths: Path[],
	ctx: CanvasRenderingContext2D | null,
	filename: string = 'rustboard.svg'
): void {
	const allShapes = [
		...rectangles.map(r => ({ type: 'rectangle', data: r, z_index: r.z_index || 0 })),
		...ellipses.map(e => ({ type: 'ellipse', data: e, z_index: e.z_index || 0 })),
		...diamonds.map(d => ({ type: 'diamond', data: d, z_index: d.z_index || 0 })),
		...lines.map(l => ({ type: 'line', data: l, z_index: l.z_index || 0 })),
		...arrows.map(a => ({ type: 'arrow', data: a, z_index: a.z_index || 0 })),
		...paths.map(p => ({ type: 'path', data: p, z_index: p.z_index || 0 })),
		...texts.map(t => ({ type: 'text', data: t, z_index: t.z_index || 0 }))
	];

	allShapes.sort((a, b) => a.z_index - b.z_index);

	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;

	rectangles.forEach(rect => {
		minX = Math.min(minX, rect.position.x);
		minY = Math.min(minY, rect.position.y);
		maxX = Math.max(maxX, rect.position.x + rect.width);
		maxY = Math.max(maxY, rect.position.y + rect.height);
	});

	ellipses.forEach(ellipse => {
		minX = Math.min(minX, ellipse.position.x - ellipse.radius_x);
		minY = Math.min(minY, ellipse.position.y - ellipse.radius_y);
		maxX = Math.max(maxX, ellipse.position.x + ellipse.radius_x);
		maxY = Math.max(maxY, ellipse.position.y + ellipse.radius_y);
	});

	diamonds.forEach(diamond => {
		minX = Math.min(minX, diamond.position.x);
		minY = Math.min(minY, diamond.position.y);
		maxX = Math.max(maxX, diamond.position.x + diamond.width);
		maxY = Math.max(maxY, diamond.position.y + diamond.height);
	});

	lines.forEach(line => {
		minX = Math.min(minX, line.start.x, line.end.x);
		minY = Math.min(minY, line.start.y, line.end.y);
		maxX = Math.max(maxX, line.start.x, line.end.x);
		maxY = Math.max(maxY, line.start.y, line.end.y);
	});

	arrows.forEach(arrow => {
		minX = Math.min(minX, arrow.start.x, arrow.end.x);
		minY = Math.min(minY, arrow.start.y, arrow.end.y);
		maxX = Math.max(maxX, arrow.start.x, arrow.end.x);
		maxY = Math.max(maxY, arrow.start.y, arrow.end.y);
	});

	paths.forEach(path => {
		const bounds = getPathBoundingBox(path);
		if (bounds) {
			minX = Math.min(minX, bounds.x);
			minY = Math.min(minY, bounds.y);
			maxX = Math.max(maxX, bounds.x + bounds.width);
			maxY = Math.max(maxY, bounds.y + bounds.height);
		}
	});

	if (ctx) {
		texts.forEach(text => {
			const contentWidth = getTextContentWidthFromBoxWidth(text.boxWidth ?? null);
			const layout = measureMultilineText(
				text.text,
				text.fontSize ?? 16,
				ctx,
				contentWidth
			);
			const horizontalPadding = TEXT_HORIZONTAL_PADDING;
			const verticalPadding = TEXT_VERTICAL_PADDING;
			const boxX = text.position.x - horizontalPadding;
			const boxY = text.position.y - layout.ascent - verticalPadding;
			const boxWidth = text.boxWidth ?? (layout.width + horizontalPadding * 2);
			const boxHeight = layout.height + verticalPadding * 2;
			minX = Math.min(minX, boxX);
			minY = Math.min(minY, boxY);
			maxX = Math.max(maxX, boxX + boxWidth);
			maxY = Math.max(maxY, boxY + boxHeight);
		});
	}

	if (minX === Infinity) {
		minX = 0;
		minY = 0;
		maxX = 100;
		maxY = 100;
	}

	const padding = 20;
	const width = maxX - minX + padding * 2;
	const height = maxY - minY + padding * 2;
	const offsetX = -minX + padding;
	const offsetY = -minY + padding;

	let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">\n`;
	svg += `<rect width="${width}" height="${height}" fill="white"/>\n`;

	allShapes.forEach(item => {
		if (item.type === 'rectangle') {
			const rect = item.data as Rectangle;
			const x = rect.position.x + offsetX;
			const y = rect.position.y + offsetY;
			const rotation = rect.rotation_angle ?? 0;
			const centerX = x + rect.width / 2;
			const centerY = y + rect.height / 2;
			const strokeColor = rect.stroke_color || '#000000';
			const fillColor = rect.fill_color || 'none';
			const lineWidth = rect.line_width || 2;

			if (rotation !== 0) {
				svg += `<g transform="translate(${centerX},${centerY}) rotate(${(rotation * 180) / Math.PI})">\n`;
				svg += `  <rect x="${-rect.width / 2}" y="${-rect.height / 2}" width="${rect.width}" height="${rect.height}" fill="${fillColor}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
				svg += `</g>\n`;
			} else {
				svg += `<rect x="${x}" y="${y}" width="${rect.width}" height="${rect.height}" fill="${fillColor}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
			}
		} else if (item.type === 'ellipse') {
			const ellipse = item.data as Ellipse;
			const cx = ellipse.position.x + offsetX;
			const cy = ellipse.position.y + offsetY;
			const rotation = ellipse.rotation_angle ?? 0;
			const strokeColor = ellipse.stroke_color || '#000000';
			const fillColor = ellipse.fill_color || 'none';
			const lineWidth = ellipse.line_width || 2;

			if (rotation !== 0) {
				svg += `<g transform="translate(${cx},${cy}) rotate(${(rotation * 180) / Math.PI})">\n`;
				svg += `  <ellipse cx="0" cy="0" rx="${ellipse.radius_x}" ry="${ellipse.radius_y}" fill="${fillColor}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
				svg += `</g>\n`;
			} else {
				svg += `<ellipse cx="${cx}" cy="${cy}" rx="${ellipse.radius_x}" ry="${ellipse.radius_y}" fill="${fillColor}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
			}
		} else if (item.type === 'diamond') {
			const diamond = item.data as Diamond;
			const x = diamond.position.x + offsetX;
			const y = diamond.position.y + offsetY;
			const centerX = x + diamond.width / 2;
			const centerY = y + diamond.height / 2;
			const halfWidth = diamond.width / 2;
			const halfHeight = diamond.height / 2;
			const rotation = diamond.rotation_angle ?? 0;
			const strokeColor = diamond.stroke_color || '#000000';
			const fillColor = diamond.fill_color || 'none';
			const lineWidth = diamond.line_width || 2;

			if (rotation !== 0) {
				svg += `<g transform="translate(${centerX},${centerY}) rotate(${(rotation * 180) / Math.PI})">\n`;
				svg += `  <polygon points="0,${-halfHeight} ${halfWidth},0 0,${halfHeight} ${-halfWidth},0" fill="${fillColor}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
				svg += `</g>\n`;
			} else {
				const points = `0,${-halfHeight} ${halfWidth},0 0,${halfHeight} ${-halfWidth},0`;
				svg += `<g transform="translate(${centerX},${centerY})">\n`;
				svg += `  <polygon points="${points}" fill="${fillColor}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
				svg += `</g>\n`;
			}
		} else if (item.type === 'line') {
			const line = item.data as Line;
			const x1 = line.start.x + offsetX;
			const y1 = line.start.y + offsetY;
			const x2 = line.end.x + offsetX;
			const y2 = line.end.y + offsetY;
			const strokeColor = line.stroke_color || '#000000';
			const lineWidth = line.line_width || 2;

			svg += `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
		} else if (item.type === 'arrow') {
			const arrow = item.data as Arrow;
			const x1 = arrow.start.x + offsetX;
			const y1 = arrow.start.y + offsetY;
			const x2 = arrow.end.x + offsetX;
			const y2 = arrow.end.y + offsetY;
			const strokeColor = arrow.stroke_color || '#000000';
			const lineWidth = arrow.line_width || 2;

			const dx = x2 - x1;
			const dy = y2 - y1;
			const angle = Math.atan2(dy, dx);
			const arrowLength = 10;
			const arrowWidth = 6;
			const arrowX1 = x2 - arrowLength * Math.cos(angle - Math.PI / 6);
			const arrowY1 = y2 - arrowLength * Math.sin(angle - Math.PI / 6);
			const arrowX2 = x2 - arrowLength * Math.cos(angle + Math.PI / 6);
			const arrowY2 = y2 - arrowLength * Math.sin(angle + Math.PI / 6);

			svg += `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${strokeColor}" stroke-width="${lineWidth}"/>\n`;
			svg += `<polygon points="${x2},${y2} ${arrowX1},${arrowY1} ${arrowX2},${arrowY2}" fill="${strokeColor}"/>\n`;
		} else if (item.type === 'path') {
			const path = item.data as Path;
			if (path.points.length > 0) {
				const strokeColor = path.stroke_color || '#000000';
				const lineWidth = path.line_width || 2;
				let pathData = `M ${path.points[0].x + offsetX} ${path.points[0].y + offsetY}`;
				for (let i = 1; i < path.points.length; i++) {
					pathData += ` L ${path.points[i].x + offsetX} ${path.points[i].y + offsetY}`;
				}
				svg += `<path d="${pathData}" fill="none" stroke="${strokeColor}" stroke-width="${lineWidth}" stroke-linecap="round" stroke-linejoin="round"/>\n`;
			}
		} else if (item.type === 'text') {
			const text = item.data as Text;
			if (!ctx) return;
			const contentWidth = getTextContentWidthFromBoxWidth(text.boxWidth ?? null);
			const layout = measureMultilineText(
				text.text,
				text.fontSize ?? 16,
				ctx,
				contentWidth
			);
			const x = text.position.x + offsetX;
			const y = text.position.y + offsetY;
			const rotation = text.rotation_angle ?? 0;
			const textColor = text.text_color || '#000000';
			const fontSize = text.fontSize ?? 16;
			const lines = layout.lines;

			if (rotation !== 0) {
				svg += `<g transform="translate(${x},${y}) rotate(${(rotation * 180) / Math.PI})">\n`;
				lines.forEach((line, index) => {
					const lineY = index * layout.lineHeight - layout.ascent;
					svg += `  <text x="0" y="${lineY}" font-family="'Lucida Console', monospace" font-size="${fontSize}" fill="${textColor}">${escapeXml(line)}</text>\n`;
				});
				svg += `</g>\n`;
			} else {
				lines.forEach((line, index) => {
					const lineY = y + index * layout.lineHeight - layout.ascent;
					svg += `<text x="${x}" y="${lineY}" font-family="'Lucida Console', monospace" font-size="${fontSize}" fill="${textColor}">${escapeXml(line)}</text>\n`;
				});
			}
		}
	});

	svg += `</svg>`;

	const blob = new Blob([svg], { type: 'image/svg+xml' });
	const url = URL.createObjectURL(blob);
	const link = document.createElement('a');
	link.download = filename;
	link.href = url;
	link.click();
	URL.revokeObjectURL(url);
}

export async function exportToPDF(
	canvas: HTMLCanvasElement,
	filename: string = 'rustboard.pdf'
): Promise<void> {
	const { default: jsPDF } = await import('jspdf');
	
	const imgData = canvas.toDataURL('image/png', 1.0);
	const imgWidth = canvas.width;
	const imgHeight = canvas.height;
	
	const pdfWidth = imgWidth * 0.75;
	const pdfHeight = imgHeight * 0.75;
	
	const pdf = new jsPDF({
		orientation: pdfWidth > pdfHeight ? 'landscape' : 'portrait',
		unit: 'pt',
		format: [pdfWidth, pdfHeight]
	});

	pdf.addImage(imgData, 'PNG', 0, 0, pdfWidth, pdfHeight);
	pdf.save(filename);
}

function escapeXml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&apos;');
}

