<script lang="ts">
	import { get } from 'svelte/store';
	import {
		selectedRectangles,
		selectedEllipses,
		selectedLines,
		selectedArrows,
		selectedDiamonds,
		selectedImages,
		selectedTexts,
		selectedPaths,
		selectedGroups,
		groups,
		editorApi,
		rectangles,
		ellipses,
		lines,
		arrows,
		diamonds,
		images,
		paths,
		texts,
		type Rectangle,
		type Ellipse,
		type Line,
		type Arrow,
		type Diamond,
		type Image,
		type Text,
		type Path
	} from '$lib/stores/editor';
	import { theme } from '$lib/stores/theme';
	import ColorPicker from './ColorPicker.svelte';
	import { saveStateToLocalStorage } from '$lib/utils/storage';
	import { tick } from 'svelte';
	import { copyToClipboard, getClipboard } from '$lib/utils/clipboard';
	import { pasteShapes } from '$lib/utils/paste-shapes';
	import { deleteShapes } from '$lib/utils/delete-shapes';
	import { sendOperation } from '$lib/utils/collaboration';
	import { edgeStyle, type EdgeStyle } from '$lib/stores/edge-style';
	import { dashPattern, type DashPattern } from '$lib/stores/dash-pattern';
	import { defaultStrokeWidth } from '$lib/stores/stroke-width';
	import { defaultStrokeColor } from '$lib/stores/stroke-color';
	import { activeTool } from '$lib/stores/tools';

	function getDefaultStrokeColor(): string {
		return $theme === 'dark' ? '#ffffff' : '#000000';
	}

	let strokeColor = '#000000';
	let fillColor: string | null = null;
	let lineWidth = 2;
	let unifiedColor = '#000000';
	let textOpacity = 1.0;

	$: displayStrokeColor = strokeColor;


	const presetStrokeAccents = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#64748b'];

	$: strokeColors = $theme === 'dark' ? ['#ffffff', ...presetStrokeAccents] : ['#000000', ...presetStrokeAccents];

	const fillColors = ['#bfdbfe', '#86efac', '#fde68a', '#fca5a5', '#cbd5e1'];

	let stylePanelRef: HTMLDivElement;

	type AlignableShape = {
		key: string;
		kind: 'rectangle' | 'ellipse' | 'diamond' | 'image' | 'text' | 'line' | 'arrow' | 'path';
		id: number;
		x: number;
		y: number;
		width: number;
		height: number;
	};

	function getRotatedRectBounds(
		x: number,
		y: number,
		width: number,
		height: number,
		rotationAngle: number | undefined,
		origin: 'center' | 'top-left' = 'top-left'
	) {
		const angle = (rotationAngle ?? 0) * Math.PI / 180;
		if (Math.abs(angle) < 1e-9) {
			return { x, y, width, height };
		}

		const cx = origin === 'center' ? x : x + width / 2;
		const cy = origin === 'center' ? y : y + height / 2;
		const halfW = width / 2;
		const halfH = height / 2;
		const corners = [
			{ x: -halfW, y: -halfH },
			{ x: halfW, y: -halfH },
			{ x: halfW, y: halfH },
			{ x: -halfW, y: halfH },
		].map((p) => ({
			x: cx + p.x * Math.cos(angle) - p.y * Math.sin(angle),
			y: cy + p.x * Math.sin(angle) + p.y * Math.cos(angle),
		}));

		const minX = Math.min(...corners.map((p) => p.x));
		const maxX = Math.max(...corners.map((p) => p.x));
		const minY = Math.min(...corners.map((p) => p.y));
		const maxY = Math.max(...corners.map((p) => p.y));
		return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
	}

	function getPathBounds(points: Array<{ x: number; y: number }>) {
		const xs = points.map((p) => p.x);
		const ys = points.map((p) => p.y);
		const minX = Math.min(...xs);
		const maxX = Math.max(...xs);
		const minY = Math.min(...ys);
		const maxY = Math.max(...ys);
		return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
	}

	function collectGroupElementIds(groupId: number, into: Set<number>) {
		const group = $groups.find((g) => g.id === groupId);
		if (!group) return;
		group.element_ids.forEach((id) => {
			const nested = $groups.find((g) => g.id === id);
			if (nested) {
				collectGroupElementIds(nested.id, into);
			} else {
				into.add(id);
			}
		});
	}

	function getAlignableSelection(): AlignableShape[] {
		const selected: AlignableShape[] = [];
		const seen = new Set<string>();
		const pushUnique = (shape: AlignableShape) => {
			const key = `${shape.kind}:${shape.id}`;
			if (seen.has(key)) return;
			seen.add(key);
			selected.push(shape);
		};

		$selectedRectangles.forEach((shape) => {
			const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
			pushUnique({ key: `rectangle:${shape.id}`, kind: 'rectangle', id: shape.id, ...bounds });
		});
		$selectedEllipses.forEach((shape) => {
			const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.radius_x * 2, shape.radius_y * 2, shape.rotation_angle, 'center');
			pushUnique({ key: `ellipse:${shape.id}`, kind: 'ellipse', id: shape.id, ...bounds });
		});
		$selectedDiamonds.forEach((shape) => {
			const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
			pushUnique({ key: `diamond:${shape.id}`, kind: 'diamond', id: shape.id, ...bounds });
		});
		$selectedLines.forEach((shape) => {
			const x = Math.min(shape.start.x, shape.end.x);
			const y = Math.min(shape.start.y, shape.end.y);
			pushUnique({ key: `line:${shape.id}`, kind: 'line', id: shape.id, x, y, width: Math.abs(shape.end.x - shape.start.x), height: Math.abs(shape.end.y - shape.start.y) });
		});
		$selectedArrows.forEach((shape) => {
			const x = Math.min(shape.start.x, shape.end.x);
			const y = Math.min(shape.start.y, shape.end.y);
			pushUnique({ key: `arrow:${shape.id}`, kind: 'arrow', id: shape.id, x, y, width: Math.abs(shape.end.x - shape.start.x), height: Math.abs(shape.end.y - shape.start.y) });
		});
		$selectedPaths.forEach((shape) => {
			if (shape.points.length === 0) return;
			const bounds = getPathBounds(shape.points);
			pushUnique({ key: `path:${shape.id}`, kind: 'path', id: shape.id, ...bounds });
		});
		$selectedImages.forEach((shape) => {
			const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
			pushUnique({ key: `image:${shape.id}`, kind: 'image', id: shape.id, ...bounds });
		});
		$selectedTexts.forEach((shape) => {
			const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
			pushUnique({ key: `text:${shape.id}`, kind: 'text', id: shape.id, ...bounds });
		});

		if ($selectedGroups.length > 0) {
			const groupElementIds = new Set<number>();
			$selectedGroups.forEach((group) => collectGroupElementIds(group.id, groupElementIds));

			$rectangles.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
				pushUnique({ key: `rectangle:${shape.id}`, kind: 'rectangle', id: shape.id, ...bounds });
			});
			$ellipses.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.radius_x * 2, shape.radius_y * 2, shape.rotation_angle, 'center');
				pushUnique({ key: `ellipse:${shape.id}`, kind: 'ellipse', id: shape.id, ...bounds });
			});
			$diamonds.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
				pushUnique({ key: `diamond:${shape.id}`, kind: 'diamond', id: shape.id, ...bounds });
			});
			$lines.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const x = Math.min(shape.start.x, shape.end.x);
				const y = Math.min(shape.start.y, shape.end.y);
				pushUnique({ key: `line:${shape.id}`, kind: 'line', id: shape.id, x, y, width: Math.abs(shape.end.x - shape.start.x), height: Math.abs(shape.end.y - shape.start.y) });
			});
			$arrows.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const x = Math.min(shape.start.x, shape.end.x);
				const y = Math.min(shape.start.y, shape.end.y);
				pushUnique({ key: `arrow:${shape.id}`, kind: 'arrow', id: shape.id, x, y, width: Math.abs(shape.end.x - shape.start.x), height: Math.abs(shape.end.y - shape.start.y) });
			});
			$paths.forEach((shape) => {
				if (!groupElementIds.has(shape.id) || shape.points.length === 0) return;
				const bounds = getPathBounds(shape.points);
				pushUnique({ key: `path:${shape.id}`, kind: 'path', id: shape.id, ...bounds });
			});
			$images.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
				pushUnique({ key: `image:${shape.id}`, kind: 'image', id: shape.id, ...bounds });
			});
			$texts.forEach((shape) => {
				if (!groupElementIds.has(shape.id)) return;
				const bounds = getRotatedRectBounds(shape.position.x, shape.position.y, shape.width, shape.height, shape.rotation_angle, 'top-left');
				pushUnique({ key: `text:${shape.id}`, kind: 'text', id: shape.id, ...bounds });
			});
		}
		return selected;
	}

	function getFillableTargets() {
		const rectangleIds = new Set<number>($selectedRectangles.map((shape) => shape.id));
		const ellipseIds = new Set<number>($selectedEllipses.map((shape) => shape.id));
		const diamondIds = new Set<number>($selectedDiamonds.map((shape) => shape.id));

		if ($selectedGroups.length > 0) {
			const groupElementIds = new Set<number>();
			$selectedGroups.forEach((group) => collectGroupElementIds(group.id, groupElementIds));
			$rectangles.forEach((shape) => {
				if (groupElementIds.has(shape.id)) rectangleIds.add(shape.id);
			});
			$ellipses.forEach((shape) => {
				if (groupElementIds.has(shape.id)) ellipseIds.add(shape.id);
			});
			$diamonds.forEach((shape) => {
				if (groupElementIds.has(shape.id)) diamondIds.add(shape.id);
			});
		}

		return {
			rectangles: Array.from(rectangleIds),
			ellipses: Array.from(ellipseIds),
			diamonds: Array.from(diamondIds)
		};
	}

	function moveAlignableShape(shape: AlignableShape, x: number, y: number) {
		if (!$editorApi) return;
			switch (shape.kind) {
			case 'rectangle':
				$editorApi.move_rectangle(BigInt(shape.id), x, y, false);
				sendOperation({ op: 'MoveRectangle', id: shape.id, position: { x, y } });
				break;
			case 'ellipse':
				$editorApi.move_ellipse(BigInt(shape.id), x + shape.width / 2, y + shape.height / 2, false);
				sendOperation({ op: 'MoveEllipse', id: shape.id, position: { x: x + shape.width / 2, y: y + shape.height / 2 } });
				break;
			case 'diamond':
				$editorApi.move_diamond(BigInt(shape.id), x, y, false);
				sendOperation({ op: 'MoveDiamond', id: shape.id, position: { x, y } });
				break;
			case 'line': {
				const current = $lines.find((line) => line.id === shape.id);
				if (!current) break;
				const deltaX = x - shape.x;
				const deltaY = y - shape.y;
				const start = { x: current.start.x + deltaX, y: current.start.y + deltaY };
				const end = { x: current.end.x + deltaX, y: current.end.y + deltaY };
				$editorApi.move_line(BigInt(shape.id), start.x, start.y, end.x, end.y, false);
				sendOperation({ op: 'MoveLine', id: shape.id, start, end });
				break;
			}
			case 'arrow': {
				const current = $arrows.find((arrow) => arrow.id === shape.id);
				if (!current) break;
				const deltaX = x - shape.x;
				const deltaY = y - shape.y;
				const start = { x: current.start.x + deltaX, y: current.start.y + deltaY };
				const end = { x: current.end.x + deltaX, y: current.end.y + deltaY };
				$editorApi.move_arrow(BigInt(shape.id), start.x, start.y, end.x, end.y, false);
				sendOperation({ op: 'MoveArrow', id: shape.id, start, end });
				break;
			}
			case 'path': {
				const deltaX = x - shape.x;
				const deltaY = y - shape.y;
				$editorApi.move_path(BigInt(shape.id), deltaX, deltaY, false);
				sendOperation({ op: 'MovePath', id: shape.id, offset_x: deltaX, offset_y: deltaY });
				break;
			}
			case 'image':
				$editorApi.move_image(BigInt(shape.id), x, y, false);
				sendOperation({ op: 'MoveImage', id: shape.id, position: { x, y } });
				break;
			case 'text':
				$editorApi.move_text(BigInt(shape.id), x, y, false);
				sendOperation({ op: 'MoveText', id: shape.id, position: { x, y } });
				break;
		}
	}

	function alignSelection(mode: 'left' | 'center' | 'right' | 'top' | 'middle' | 'bottom') {
		if (!$editorApi) return;
		const selected = getAlignableSelection();
		if (selected.length < 2) return;

		const minX = Math.min(...selected.map((shape) => shape.x));
		const maxX = Math.max(...selected.map((shape) => shape.x + shape.width));
		const minY = Math.min(...selected.map((shape) => shape.y));
		const maxY = Math.max(...selected.map((shape) => shape.y + shape.height));
		const centerX = (minX + maxX) / 2;
		const centerY = (minY + maxY) / 2;

		selected.forEach((shape) => {
			let x = shape.x;
			let y = shape.y;
			if (mode === 'left') x = minX;
			if (mode === 'center') x = centerX - shape.width / 2;
			if (mode === 'right') x = maxX - shape.width;
			if (mode === 'top') y = minY;
			if (mode === 'middle') y = centerY - shape.height / 2;
			if (mode === 'bottom') y = maxY - shape.height;
			moveAlignableShape(shape, x, y);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function distributeSelection(axis: 'horizontal' | 'vertical') {
		if (!$editorApi) return;
		const selected = getAlignableSelection();
		if (selected.length < 3) return;

		const sorted = [...selected].sort((a, b) => (axis === 'horizontal' ? a.x - b.x : a.y - b.y));
		const first = sorted[0];
		const last = sorted[sorted.length - 1];
		const span = axis === 'horizontal' ? last.x - first.x : last.y - first.y;
		if (span <= 0) return;
		const step = span / (sorted.length - 1);

		sorted.forEach((shape, index) => {
			if (shape.key === first.key || shape.key === last.key) return;
			const x = axis === 'horizontal' ? first.x + step * index : shape.x;
			const y = axis === 'vertical' ? first.y + step * index : shape.y;
			moveAlignableShape(shape, x, y);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function openStrokeColorPicker(event: MouseEvent) {
		const button = event.currentTarget as HTMLButtonElement;
		const buttonRect = button.getBoundingClientRect();
		const pickerLeft = Math.max(8, Math.min(window.innerWidth - 40, buttonRect.left - 24));
		const pickerTop = Math.max(8, Math.min(window.innerHeight - 40, buttonRect.top));
		
		const input = document.createElement('input');
		input.type = 'color';
		input.value = displayStrokeColor;
		input.style.position = 'fixed';
		input.style.left = `${pickerLeft}px`;
		input.style.top = `${pickerTop}px`;
		input.style.width = '1px';
		input.style.height = '1px';
		input.style.opacity = '0.01';
		input.style.pointerEvents = 'auto';
		input.style.zIndex = '9999';
		document.body.appendChild(input);
		
		setTimeout(() => {
			input.focus();
			input.click();
		}, 10);
		
		input.onchange = (e) => {
			const selectedColor = (e.target as HTMLInputElement).value;
			const denormalizedColor = $theme === 'dark' && selectedColor === '#000000' ? '#ffffff' : 
			                          $theme !== 'dark' && selectedColor === '#ffffff' ? '#000000' : selectedColor;
			updateStrokeColor(denormalizedColor);
			if (document.body.contains(input)) {
				document.body.removeChild(input);
			}
		};
		input.onblur = () => {
			setTimeout(() => {
				if (document.body.contains(input)) {
					document.body.removeChild(input);
				}
			}, 100);
		};
	}

	function openFillColorPicker(event: MouseEvent) {
		const button = event.currentTarget as HTMLButtonElement;
		const buttonRect = button.getBoundingClientRect();
		const pickerLeft = Math.max(8, Math.min(window.innerWidth - 40, buttonRect.left - 24));
		const pickerTop = Math.max(8, Math.min(window.innerHeight - 40, buttonRect.top));
		
		const input = document.createElement('input');
		input.type = 'color';
		input.value = fillColor || '#000000';
		input.style.position = 'fixed';
		input.style.left = `${pickerLeft}px`;
		input.style.top = `${pickerTop}px`;
		input.style.width = '1px';
		input.style.height = '1px';
		input.style.opacity = '0.01';
		input.style.pointerEvents = 'auto';
		input.style.zIndex = '9999';
		document.body.appendChild(input);
		
		setTimeout(() => {
			input.focus();
			input.click();
		}, 10);
		
		input.onchange = (e) => {
			updateFillColor((e.target as HTMLInputElement).value);
			if (document.body.contains(input)) {
				document.body.removeChild(input);
			}
		};
		input.onblur = () => {
			setTimeout(() => {
				if (document.body.contains(input)) {
					document.body.removeChild(input);
				}
			}, 100);
		};
	}

	function isLightColor(hexColor: string): boolean {
		const hex = hexColor.replace('#', '');
		const r = parseInt(hex.slice(0, 2), 16);
		const g = parseInt(hex.slice(2, 4), 16);
		const b = parseInt(hex.slice(4, 6), 16);
		const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
		return luminance > 0.5;
	}

	let isUpdatingColor = false;

	$: {
		if (!isUpdatingColor && $theme) {
			const totalSelected =
				$selectedRectangles.length +
				$selectedEllipses.length +
				$selectedDiamonds.length +
				$selectedLines.length +
				$selectedArrows.length +
				$selectedImages.length +
				$selectedTexts.length +
				$selectedPaths.length;

			if (totalSelected === 0 && $selectedGroups.length === 0) {
				strokeColor = getDefaultStrokeColor();
				fillColor = null;
				lineWidth = 2;
				textOpacity = 1.0;
			} else {
				const shapes: Array<Rectangle | Ellipse | Line | Arrow | Diamond | Text | Path> = [
					...$selectedRectangles,
					...$selectedEllipses,
					...$selectedDiamonds,
					...$selectedLines,
					...$selectedArrows,
					...$selectedTexts,
					...$selectedPaths
				];

				const strokeColors = shapes
					.map((s) => {
						if ((s as any).color !== undefined) {
							return (s as any).color || getDefaultStrokeColor();
						}
						return (s as any).stroke_color || getDefaultStrokeColor();
					})
					.filter((c, i, arr) => arr.indexOf(c) === i);
				if (strokeColors.length === 1) {
					let newStrokeColor = strokeColors[0];

					const oppositeThemeDefault = $theme === 'dark' ? '#000000' : '#ffffff';
					const currentThemeDefault = getDefaultStrokeColor();

					if (newStrokeColor === oppositeThemeDefault) {
						newStrokeColor = currentThemeDefault;
					}

					if (newStrokeColor !== strokeColor) {
						strokeColor = newStrokeColor;
					}
				}

				const fillColors = shapes
					.map((s) => (s as any).fill_color ?? null)
					.filter((c, i, arr) => arr.indexOf(c) === i);
				fillColor = fillColors.length === 1 ? fillColors[0] : null;

				const nonTextShapes = shapes.filter((s) => (s as any).color === undefined);
				const lineWidths = nonTextShapes
					.map((s) => (s as any).line_width ?? 2)
					.filter((w, i, arr) => arr.indexOf(w) === i);
				lineWidth = lineWidths.length === 1 ? lineWidths[0] : 2;
				if (lineWidths.length === 1 && (lineWidths[0] === 1 || lineWidths[0] === 2 || lineWidths[0] === 4)) {
					defaultStrokeWidth.set(lineWidths[0]);
				}

				if ($selectedTexts.length > 0) {
					const opacities = $selectedTexts
						.map((t) => (t as any).opacity ?? 1.0)
						.filter((o, i, arr) => arr.indexOf(o) === i);
					textOpacity = opacities.length === 1 ? opacities[0] : 1.0;
				} else {
					textOpacity = 1.0;
				}

			}
		}
	}


	async function updateStrokeColor(color: string) {
		isUpdatingColor = true;
		strokeColor = color;
		defaultStrokeColor.set(color);
		if (!$editorApi) {
			isUpdatingColor = false;
			return;
		}

		const selectedRects = get(selectedRectangles);
		const selectedElls = get(selectedEllipses);
		const selectedDias = get(selectedDiamonds);
		const selectedLns = get(selectedLines);
		const selectedArrs = get(selectedArrows);
		const selectedPths = get(selectedPaths);
		const selectedTxts = get(selectedTexts);

		selectedRects.forEach((rect: Rectangle) => {
			$editorApi.set_rectangle_stroke_color(BigInt(rect.id), color, false);
			sendOperation({ op: 'SetRectangleStyle', id: rect.id, stroke_color: color });
		});
		selectedElls.forEach((ellipse: Ellipse) => {
			$editorApi.set_ellipse_stroke_color(BigInt(ellipse.id), color, false);
			sendOperation({ op: 'SetEllipseStyle', id: ellipse.id, stroke_color: color });
		});
		selectedDias.forEach((diamond: Diamond) => {
			$editorApi.set_diamond_stroke_color(BigInt(diamond.id), color, false);
			sendOperation({ op: 'SetDiamondStyle', id: diamond.id, stroke_color: color });
		});
		selectedLns.forEach((line: Line) => {
			$editorApi.set_line_stroke_color(BigInt(line.id), color, false);
			sendOperation({ op: 'SetLineStyle', id: line.id, stroke_color: color });
		});
		selectedArrs.forEach((arrow: Arrow) => {
			$editorApi.set_arrow_stroke_color(BigInt(arrow.id), color, false);
			sendOperation({ op: 'SetArrowStyle', id: arrow.id, stroke_color: color });
		});
		selectedPths.forEach((path: Path) => {
			$editorApi.set_path_stroke_color(BigInt(path.id), color, false);
			sendOperation({ op: 'SetPathStyle', id: path.id, stroke_color: color });
		});
		selectedTxts.forEach((text: Text) => {
			$editorApi.set_text_color(BigInt(text.id), color, false);
			sendOperation({ op: 'SetTextStyle', id: text.id, color });
		});

		$editorApi.save_snapshot();
		updateStores();
		await tick();
		isUpdatingColor = false;
		saveStateToLocalStorage();
	}

	async function updateFillColor(color: string | null) {
		isUpdatingColor = true;
		fillColor = color;
		if (!$editorApi) {
			isUpdatingColor = false;
			return;
		}

		const colorValue = color || null;
		const fillTargets = getFillableTargets();

		fillTargets.rectangles.forEach((id) => {
			$editorApi.set_rectangle_fill_color(BigInt(id), colorValue, false);
			sendOperation({ op: 'SetRectangleStyle', id, fill_color: colorValue });
		});
		fillTargets.ellipses.forEach((id) => {
			$editorApi.set_ellipse_fill_color(BigInt(id), colorValue, false);
			sendOperation({ op: 'SetEllipseStyle', id, fill_color: colorValue });
		});
		fillTargets.diamonds.forEach((id) => {
			$editorApi.set_diamond_fill_color(BigInt(id), colorValue, false);
			sendOperation({ op: 'SetDiamondStyle', id, fill_color: colorValue });
		});

		$editorApi.save_snapshot();
		updateStores();
		await tick();
		isUpdatingColor = false;
		saveStateToLocalStorage();
	}

	function updateLineWidth(width: number) {
		lineWidth = Math.max(0.1, width);
		defaultStrokeWidth.set(lineWidth);
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_line_width(BigInt(rect.id), lineWidth, false);
			sendOperation({ op: 'SetRectangleStyle', id: rect.id, line_width: lineWidth });
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_line_width(BigInt(ellipse.id), lineWidth, false);
			sendOperation({ op: 'SetEllipseStyle', id: ellipse.id, line_width: lineWidth });
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_line_width(BigInt(diamond.id), lineWidth, false);
			sendOperation({ op: 'SetDiamondStyle', id: diamond.id, line_width: lineWidth });
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_line_line_width(BigInt(line.id), lineWidth, false);
			sendOperation({ op: 'SetLineStyle', id: line.id, line_width: lineWidth });
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_arrow_line_width(BigInt(arrow.id), lineWidth, false);
			sendOperation({ op: 'SetArrowStyle', id: arrow.id, line_width: lineWidth });
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_path_line_width(BigInt(path.id), lineWidth, false);
			sendOperation({ op: 'SetPathStyle', id: path.id, line_width: lineWidth });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}


	function updateFontSize(fontSize: number) {
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_font_size(BigInt(text.id), fontSize, false);
			sendOperation({ op: 'SetTextStyle', id: text.id, font_size: fontSize });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateFontWeight(fontWeight: string) {
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_font_weight(BigInt(text.id), fontWeight, false);
			sendOperation({ op: 'SetTextStyle', id: text.id, font_weight: fontWeight });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateTextAlign(textAlign: string) {
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_text_align(BigInt(text.id), textAlign, false);
			sendOperation({ op: 'SetTextStyle', id: text.id, text_align: textAlign });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateTextOpacity(opacity: number) {
		textOpacity = Math.max(0, Math.min(1, opacity));
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_opacity(BigInt(text.id), textOpacity, false);
			sendOperation({ op: 'SetTextStyle', id: text.id, opacity: textOpacity });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateStrokeWidthType(width: number) {
		lineWidth = width;
		defaultStrokeWidth.set(width);
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_line_width(BigInt(rect.id), width, false);
			sendOperation({ op: 'SetRectangleStyle', id: rect.id, line_width: width });
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_line_width(BigInt(ellipse.id), width, false);
			sendOperation({ op: 'SetEllipseStyle', id: ellipse.id, line_width: width });
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_line_width(BigInt(diamond.id), width, false);
			sendOperation({ op: 'SetDiamondStyle', id: diamond.id, line_width: width });
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_line_line_width(BigInt(line.id), width, false);
			sendOperation({ op: 'SetLineStyle', id: line.id, line_width: width });
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_arrow_line_width(BigInt(arrow.id), width, false);
			sendOperation({ op: 'SetArrowStyle', id: arrow.id, line_width: width });
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_path_line_width(BigInt(path.id), width, false);
			sendOperation({ op: 'SetPathStyle', id: path.id, line_width: width });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}


	function bringToFront() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => { $editorApi!.bring_shape_to_front(BigInt(r.id)); sendOperation({ op: 'BringToFront', id: r.id }); });
		$selectedEllipses.forEach(e => { $editorApi!.bring_shape_to_front(BigInt(e.id)); sendOperation({ op: 'BringToFront', id: e.id }); });
		$selectedDiamonds.forEach(d => { $editorApi!.bring_shape_to_front(BigInt(d.id)); sendOperation({ op: 'BringToFront', id: d.id }); });
		$selectedLines.forEach(l => { $editorApi!.bring_shape_to_front(BigInt(l.id)); sendOperation({ op: 'BringToFront', id: l.id }); });
		$selectedArrows.forEach(a => { $editorApi!.bring_shape_to_front(BigInt(a.id)); sendOperation({ op: 'BringToFront', id: a.id }); });
		$selectedPaths.forEach(p => { $editorApi!.bring_shape_to_front(BigInt(p.id)); sendOperation({ op: 'BringToFront', id: p.id }); });
		$selectedImages.forEach(i => { $editorApi!.bring_shape_to_front(BigInt(i.id)); sendOperation({ op: 'BringToFront', id: i.id }); });
		$selectedTexts.forEach(t => { $editorApi!.bring_shape_to_front(BigInt(t.id)); sendOperation({ op: 'BringToFront', id: t.id }); });
		updateStores();
		saveStateToLocalStorage();
	}

	function bringForward() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => { $editorApi!.bring_shape_forward(BigInt(r.id)); sendOperation({ op: 'BringForward', id: r.id }); });
		$selectedEllipses.forEach(e => { $editorApi!.bring_shape_forward(BigInt(e.id)); sendOperation({ op: 'BringForward', id: e.id }); });
		$selectedDiamonds.forEach(d => { $editorApi!.bring_shape_forward(BigInt(d.id)); sendOperation({ op: 'BringForward', id: d.id }); });
		$selectedLines.forEach(l => { $editorApi!.bring_shape_forward(BigInt(l.id)); sendOperation({ op: 'BringForward', id: l.id }); });
		$selectedArrows.forEach(a => { $editorApi!.bring_shape_forward(BigInt(a.id)); sendOperation({ op: 'BringForward', id: a.id }); });
		$selectedPaths.forEach(p => { $editorApi!.bring_shape_forward(BigInt(p.id)); sendOperation({ op: 'BringForward', id: p.id }); });
		$selectedImages.forEach(i => { $editorApi!.bring_shape_forward(BigInt(i.id)); sendOperation({ op: 'BringForward', id: i.id }); });
		$selectedTexts.forEach(t => { $editorApi!.bring_shape_forward(BigInt(t.id)); sendOperation({ op: 'BringForward', id: t.id }); });
		updateStores();
		saveStateToLocalStorage();
	}

	function sendBackward() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => { $editorApi!.send_shape_backward(BigInt(r.id)); sendOperation({ op: 'SendBackward', id: r.id }); });
		$selectedEllipses.forEach(e => { $editorApi!.send_shape_backward(BigInt(e.id)); sendOperation({ op: 'SendBackward', id: e.id }); });
		$selectedDiamonds.forEach(d => { $editorApi!.send_shape_backward(BigInt(d.id)); sendOperation({ op: 'SendBackward', id: d.id }); });
		$selectedLines.forEach(l => { $editorApi!.send_shape_backward(BigInt(l.id)); sendOperation({ op: 'SendBackward', id: l.id }); });
		$selectedArrows.forEach(a => { $editorApi!.send_shape_backward(BigInt(a.id)); sendOperation({ op: 'SendBackward', id: a.id }); });
		$selectedPaths.forEach(p => { $editorApi!.send_shape_backward(BigInt(p.id)); sendOperation({ op: 'SendBackward', id: p.id }); });
		$selectedImages.forEach(i => { $editorApi!.send_shape_backward(BigInt(i.id)); sendOperation({ op: 'SendBackward', id: i.id }); });
		$selectedTexts.forEach(t => { $editorApi!.send_shape_backward(BigInt(t.id)); sendOperation({ op: 'SendBackward', id: t.id }); });
		updateStores();
		saveStateToLocalStorage();
	}

	function sendToBack() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => { $editorApi!.send_shape_to_back(BigInt(r.id)); sendOperation({ op: 'SendToBack', id: r.id }); });
		$selectedEllipses.forEach(e => { $editorApi!.send_shape_to_back(BigInt(e.id)); sendOperation({ op: 'SendToBack', id: e.id }); });
		$selectedDiamonds.forEach(d => { $editorApi!.send_shape_to_back(BigInt(d.id)); sendOperation({ op: 'SendToBack', id: d.id }); });
		$selectedLines.forEach(l => { $editorApi!.send_shape_to_back(BigInt(l.id)); sendOperation({ op: 'SendToBack', id: l.id }); });
		$selectedArrows.forEach(a => { $editorApi!.send_shape_to_back(BigInt(a.id)); sendOperation({ op: 'SendToBack', id: a.id }); });
		$selectedPaths.forEach(p => { $editorApi!.send_shape_to_back(BigInt(p.id)); sendOperation({ op: 'SendToBack', id: p.id }); });
		$selectedImages.forEach(i => { $editorApi!.send_shape_to_back(BigInt(i.id)); sendOperation({ op: 'SendToBack', id: i.id }); });
		$selectedTexts.forEach(t => { $editorApi!.send_shape_to_back(BigInt(t.id)); sendOperation({ op: 'SendToBack', id: t.id }); });
		updateStores();
		saveStateToLocalStorage();
	}

	function updateStores() {
		if (!$editorApi) return;
		const api = get(editorApi);
		if (!api) return;
		
		const selectedRectIds = new Set($selectedRectangles.map(r => r.id));
		const selectedEllipseIds = new Set($selectedEllipses.map(e => e.id));
		const selectedLineIds = new Set($selectedLines.map(l => l.id));
		const selectedArrowIds = new Set($selectedArrows.map(a => a.id));
		const selectedDiamondIds = new Set($selectedDiamonds.map(d => d.id));
		const selectedImageIds = new Set($selectedImages.map(i => i.id));
		const selectedPathIds = new Set($selectedPaths.map(p => p.id));
		
		const allRectangles = api.get_rectangles() as Rectangle[];
		const allEllipses = api.get_ellipses() as Ellipse[];
		const allLines = api.get_lines() as Line[];
		const allArrows = api.get_arrows() as Arrow[];
		const allDiamonds = api.get_diamonds() as Diamond[];
		const allImages = api.get_images() as Image[];
		const allPaths = api.get_paths() as Path[];
		const allTexts = api.get_texts() as Text[];
		
		rectangles.set(allRectangles);
		ellipses.set(allEllipses);
		lines.set(allLines);
		arrows.set(allArrows);
		diamonds.set(allDiamonds);
		images.set(allImages);
		paths.set(allPaths);
		texts.set(allTexts);
		
		selectedRectangles.set(allRectangles.filter(r => selectedRectIds.has(r.id)));
		selectedEllipses.set(allEllipses.filter(e => selectedEllipseIds.has(e.id)));
		selectedLines.set(allLines.filter(l => selectedLineIds.has(l.id)));
		selectedArrows.set(allArrows.filter(a => selectedArrowIds.has(a.id)));
		selectedDiamonds.set(allDiamonds.filter(d => selectedDiamondIds.has(d.id)));
		selectedImages.set(allImages.filter(i => selectedImageIds.has(i.id)));
		selectedPaths.set(allPaths.filter(p => selectedPathIds.has(p.id)));
		selectedTexts.set(allTexts.filter(t => {
			const selectedIds = new Set($selectedTexts.map(st => st.id));
			return selectedIds.has(t.id);
		}));
	}

	$: hasSelection =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedImages.length > 0 ||
		$selectedTexts.length > 0 ||
		$selectedPaths.length > 0;

	$: hasFillableShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		getFillableTargets().rectangles.length > 0 ||
		getFillableTargets().ellipses.length > 0 ||
		getFillableTargets().diamonds.length > 0;

	$: hasEditableEdges =
		$selectedRectangles.length > 0 ||
		$selectedDiamonds.length > 0;

	$: hasDashPatternShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedPaths.length > 0 ||
		$selectedImages.length > 0;

	$: showDashPatternControls =
		hasDashPatternShapes ||
		$activeTool === 'rectangle' ||
		$activeTool === 'ellipse' ||
		$activeTool === 'diamond' ||
		$activeTool === 'line' ||
		$activeTool === 'arrow' ||
		$activeTool === 'freehand';

	$: hasImagesOnly = $selectedImages.length > 0 &&
		$selectedRectangles.length === 0 &&
		$selectedEllipses.length === 0 &&
		$selectedDiamonds.length === 0 &&
		$selectedLines.length === 0 &&
		$selectedArrows.length === 0 &&
		$selectedTexts.length === 0 &&
		$selectedPaths.length === 0;

	$: hasTexts = $selectedTexts.length > 0;

	$: hasShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedTexts.length > 0 ||
		$selectedPaths.length > 0;

	$: hasStrokeWidthControls =
		(hasShapes || $activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond' || $activeTool === 'line' || $activeTool === 'arrow' || $activeTool === 'freehand') &&
		$selectedTexts.length === 0;

	$: showStrokeColors =
		!hasImagesOnly &&
		(hasShapes || $activeTool === 'freehand' || $activeTool === 'line' || $activeTool === 'arrow');

	$: compactFreehandStrokeRow =
		hasStrokeWidthControls &&
		($activeTool === 'freehand' || $activeTool === 'line' || $activeTool === 'arrow') &&
		!hasFillableShapes &&
		showStrokeColors;

	$: isSingleSelection =
		($selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length +
		 $selectedLines.length + $selectedArrows.length + $selectedTexts.length + $selectedPaths.length) === 1;

	async function updateUnifiedColor(color: string) {
		unifiedColor = color;
		if (!$editorApi) return;

		const selectedRects = get(selectedRectangles);
		const selectedElls = get(selectedEllipses);
		const selectedDias = get(selectedDiamonds);
		const selectedLns = get(selectedLines);
		const selectedArrs = get(selectedArrows);
		const selectedPths = get(selectedPaths);
		selectedRects.forEach((rect: Rectangle) => {
			$editorApi.set_rectangle_stroke_color(BigInt(rect.id), color, false);
			sendOperation({ op: 'SetRectangleStyle', id: rect.id, stroke_color: color });
		});
		selectedElls.forEach((ellipse: Ellipse) => {
			$editorApi.set_ellipse_stroke_color(BigInt(ellipse.id), color, false);
			sendOperation({ op: 'SetEllipseStyle', id: ellipse.id, stroke_color: color });
		});
		selectedDias.forEach((diamond: Diamond) => {
			$editorApi.set_diamond_stroke_color(BigInt(diamond.id), color, false);
			sendOperation({ op: 'SetDiamondStyle', id: diamond.id, stroke_color: color });
		});
		selectedLns.forEach((line: Line) => {
			$editorApi.set_line_stroke_color(BigInt(line.id), color, false);
			sendOperation({ op: 'SetLineStyle', id: line.id, stroke_color: color });
		});
		selectedArrs.forEach((arrow: Arrow) => {
			$editorApi.set_arrow_stroke_color(BigInt(arrow.id), color, false);
			sendOperation({ op: 'SetArrowStyle', id: arrow.id, stroke_color: color });
		});
		selectedPths.forEach((path: Path) => {
			$editorApi.set_path_stroke_color(BigInt(path.id), color, false);
			sendOperation({ op: 'SetPathStyle', id: path.id, stroke_color: color });
		});

		$editorApi.save_snapshot();
		updateStores();
		await tick();
		saveStateToLocalStorage();
	}

	function handleDuplicate() {
		if (!$editorApi) return;

		const hasSelection = $selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0 || $selectedTexts.length > 0 || $selectedPaths.length > 0 || $selectedImages.length > 0;
		if (!hasSelection) return;

		const selectedIds = new Set([
			...$selectedRectangles.map(r => r.id),
			...$selectedEllipses.map(e => e.id),
			...$selectedLines.map(l => l.id),
			...$selectedArrows.map(a => a.id),
			...$selectedDiamonds.map(d => d.id),
			...$selectedTexts.map(t => t.id),
			...$selectedPaths.map(p => p.id),
			...$selectedImages.map(i => i.id)
		]);

		const allRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
		const allEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
		const allLines = Array.from($editorApi.get_lines() as Line[]);
		const allArrows = Array.from($editorApi.get_arrows() as Arrow[]);
		const allDiamonds = Array.from($editorApi.get_diamonds() as Diamond[]);
		const allTexts = Array.from($editorApi.get_texts() as Text[]);
		const allPaths = Array.from($editorApi.get_paths() as Path[]);
		const allImages = Array.from($editorApi.get_images() as Image[]);

		const currentRectangles = allRectangles.filter(r => selectedIds.has(r.id));
		const currentEllipses = allEllipses.filter(e => selectedIds.has(e.id));
		const currentLines = allLines.filter(l => selectedIds.has(l.id));
		const currentArrows = allArrows.filter(a => selectedIds.has(a.id));
		const currentDiamonds = allDiamonds.filter(d => selectedIds.has(d.id));
		const currentTexts = allTexts.filter((t: Text) => selectedIds.has(t.id));
		const currentPaths = allPaths.filter(p => selectedIds.has(p.id));
		const currentImages = allImages.filter(i => selectedIds.has(i.id));

		copyToClipboard(currentRectangles, currentEllipses, currentLines, currentArrows, currentDiamonds, currentImages, currentTexts, currentPaths);
		const clipboard = getClipboard();

		const bounds: Array<{ minX: number; minY: number }> = [];
		clipboard.rectangles.forEach(r => bounds.push({ minX: r.position.x, minY: r.position.y }));
		clipboard.ellipses.forEach(e => bounds.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y }));
		clipboard.diamonds.forEach(d => bounds.push({ minX: d.position.x, minY: d.position.y }));
		clipboard.lines.forEach(l => bounds.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) }));
		clipboard.arrows.forEach(a => bounds.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) }));
		clipboard.images.forEach(i => bounds.push({ minX: i.position.x, minY: i.position.y }));
		clipboard.texts.forEach(t => bounds.push({ minX: t.position.x, minY: t.position.y }));
		clipboard.paths.forEach(p => {
			if (p.points.length > 0) {
				const minX = Math.min(...p.points.map(pt => pt.x));
				const minY = Math.min(...p.points.map(pt => pt.y));
				bounds.push({ minX, minY });
			}
		});

		if (bounds.length === 0) return;

		const minX = Math.min(...bounds.map(b => b.minX));
		const minY = Math.min(...bounds.map(b => b.minY));
		const duplicateX = minX + 20;
		const duplicateY = minY + 20;

		pasteShapes(clipboard, duplicateX, duplicateY);
	}

	function handleDelete() {
		if (!$editorApi) return;

		const rectangleIds = $selectedRectangles.map(rect => rect.id);
		const ellipseIds = $selectedEllipses.map(ellipse => ellipse.id);
		const diamondIds = $selectedDiamonds.map(diamond => diamond.id);
		const lineIds = $selectedLines.map(line => line.id);
		const arrowIds = $selectedArrows.map(arrow => arrow.id);
		const pathIds = $selectedPaths.map(path => path.id);
		const imageIds = $selectedImages.map(image => image.id);
		const textIds = $selectedTexts.map(text => text.id);

		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds, textIds, pathIds, imageIds);
	}

	function handleLock() {
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_element_locked(BigInt(rect.id), true, false);
			sendOperation({ op: 'SetElementLock', id: rect.id, locked: true });
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_element_locked(BigInt(ellipse.id), true, false);
			sendOperation({ op: 'SetElementLock', id: ellipse.id, locked: true });
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_element_locked(BigInt(diamond.id), true, false);
			sendOperation({ op: 'SetElementLock', id: diamond.id, locked: true });
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_element_locked(BigInt(line.id), true, false);
			sendOperation({ op: 'SetElementLock', id: line.id, locked: true });
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_element_locked(BigInt(arrow.id), true, false);
			sendOperation({ op: 'SetElementLock', id: arrow.id, locked: true });
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_element_locked(BigInt(path.id), true, false);
			sendOperation({ op: 'SetElementLock', id: path.id, locked: true });
		});
		$selectedImages.forEach((image) => {
			$editorApi.set_element_locked(BigInt(image.id), true, false);
			sendOperation({ op: 'SetElementLock', id: image.id, locked: true });
		});
		$selectedTexts.forEach((text) => {
			$editorApi.set_element_locked(BigInt(text.id), true, false);
			sendOperation({ op: 'SetElementLock', id: text.id, locked: true });
		});
		$selectedGroups.forEach((group) => {
			$editorApi.set_element_locked(BigInt(group.id), true, false);
			sendOperation({ op: 'SetElementLock', id: group.id, locked: true });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function handleUnlock() {
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_element_locked(BigInt(rect.id), false, false);
			sendOperation({ op: 'SetElementLock', id: rect.id, locked: false });
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_element_locked(BigInt(ellipse.id), false, false);
			sendOperation({ op: 'SetElementLock', id: ellipse.id, locked: false });
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_element_locked(BigInt(diamond.id), false, false);
			sendOperation({ op: 'SetElementLock', id: diamond.id, locked: false });
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_element_locked(BigInt(line.id), false, false);
			sendOperation({ op: 'SetElementLock', id: line.id, locked: false });
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_element_locked(BigInt(arrow.id), false, false);
			sendOperation({ op: 'SetElementLock', id: arrow.id, locked: false });
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_element_locked(BigInt(path.id), false, false);
			sendOperation({ op: 'SetElementLock', id: path.id, locked: false });
		});
		$selectedImages.forEach((image) => {
			$editorApi.set_element_locked(BigInt(image.id), false, false);
			sendOperation({ op: 'SetElementLock', id: image.id, locked: false });
		});
		$selectedTexts.forEach((text) => {
			$editorApi.set_element_locked(BigInt(text.id), false, false);
			sendOperation({ op: 'SetElementLock', id: text.id, locked: false });
		});
		$selectedGroups.forEach((group) => {
			$editorApi.set_element_locked(BigInt(group.id), false, false);
			sendOperation({ op: 'SetElementLock', id: group.id, locked: false });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	$: allLocked = $editorApi ? (() => {
		const allSelected = [
			...$selectedRectangles.map(r => ({ id: r.id })),
			...$selectedEllipses.map(e => ({ id: e.id })),
			...$selectedDiamonds.map(d => ({ id: d.id })),
			...$selectedLines.map(l => ({ id: l.id })),
			...$selectedArrows.map(a => ({ id: a.id })),
			...$selectedPaths.map(p => ({ id: p.id })),
			...$selectedImages.map(i => ({ id: i.id })),
			...$selectedTexts.map(t => ({ id: t.id })),
			...$selectedGroups.map(g => ({ id: g.id }))
		];
		if (allSelected.length === 0) return false;
		return allSelected.every(el => $editorApi.is_element_locked(BigInt(el.id)));
	})() : false;

	function handleEdgeStyleChange(style: EdgeStyle) {
		if (!$editorApi) return;

		edgeStyle.set(style);
		const radius = style === 'rounded' ? 4.0 : 0.0;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_border_radius(BigInt(rect.id), radius, false);
			sendOperation({ op: 'SetRectangleStyle', id: rect.id, border_radius: radius });
		});

		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_border_radius(BigInt(diamond.id), radius, false);
			sendOperation({ op: 'SetDiamondStyle', id: diamond.id, border_radius: radius });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function handleDashPatternChange(pattern: DashPattern) {
		if (!$editorApi) return;

		dashPattern.set(pattern);
		const patternStr = pattern;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_dash_pattern(BigInt(rect.id), patternStr, false);
			sendOperation({ op: 'SetRectangleStyle', id: rect.id, dash_pattern: patternStr });
		});

		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_dash_pattern(BigInt(ellipse.id), patternStr, false);
			sendOperation({ op: 'SetEllipseStyle', id: ellipse.id, dash_pattern: patternStr });
		});

		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_dash_pattern(BigInt(diamond.id), patternStr, false);
			sendOperation({ op: 'SetDiamondStyle', id: diamond.id, dash_pattern: patternStr });
		});

		$selectedLines.forEach((line) => {
			$editorApi.set_line_dash_pattern(BigInt(line.id), patternStr, false);
			sendOperation({ op: 'SetLineStyle', id: line.id, dash_pattern: patternStr });
		});

		$selectedArrows.forEach((arrow) => {
			$editorApi.set_arrow_dash_pattern(BigInt(arrow.id), patternStr, false);
			sendOperation({ op: 'SetArrowStyle', id: arrow.id, dash_pattern: patternStr });
		});

		$selectedPaths.forEach((path) => {
			$editorApi.set_path_dash_pattern(BigInt(path.id), patternStr, false);
			sendOperation({ op: 'SetPathStyle', id: path.id, dash_pattern: patternStr });
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}


	$: {
		if (hasEditableEdges) {
			const allBorderRadii: number[] = [];
			$selectedRectangles.forEach(r => {
				const br = r.border_radius ?? 0;
				allBorderRadii.push(br);
			});
			$selectedDiamonds.forEach(d => {
				const br = d.border_radius ?? 0;
				allBorderRadii.push(br);
			});

			const allSharp = allBorderRadii.length > 0 && allBorderRadii.every(r => r === 0);
			const allRounded = allBorderRadii.length > 0 && allBorderRadii.every(r => r > 0);

			if (allSharp) {
				edgeStyle.set('sharp');
			} else if (allRounded) {
				edgeStyle.set('rounded');
			}
		}
	}

	$: {
		if (hasDashPatternShapes) {
			const allDashPatterns: string[] = [];
			$selectedRectangles.forEach(r => {
				const dp = r.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedEllipses.forEach(e => {
				const dp = e.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedDiamonds.forEach(d => {
				const dp = d.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedLines.forEach(l => {
				const dp = l.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedArrows.forEach(a => {
				const dp = a.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedPaths.forEach(p => {
				const dp = p.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});

			const allSolid = allDashPatterns.length > 0 && allDashPatterns.every(p => p === 'solid');
			const allDashed = allDashPatterns.length > 0 && allDashPatterns.every(p => p === 'dashed');
			const allDotted = allDashPatterns.length > 0 && allDashPatterns.every(p => p === 'dotted');

			if (allSolid) {
				dashPattern.set('solid');
			} else if (allDashed) {
				dashPattern.set('dashed');
			} else if (allDotted) {
				dashPattern.set('dotted');
			}
		}
	}</script>

{#if hasSelection}
	<div bind:this={stylePanelRef} class={`fixed right-1.5 bottom-24 left-1.5 z-50 max-h-[40vh] overflow-y-auto overscroll-contain backdrop-blur-sm border rounded-lg p-2.5 md:absolute md:top-2 md:right-2 md:bottom-auto md:left-auto md:max-h-[calc(100vh-1rem)] md:w-[240px] md:min-w-[240px] md:p-3 ${$theme === 'dark' ? 'bg-stone-800/95 border-stone-700/50' : 'bg-white/95 border-stone-200/50'} shadow-lg`}>
		<div class="space-y-2 min-w-0 md:space-y-2.5">
			<div class="space-y-1.5">
				<div class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Arrange</div>
				<div class="grid grid-cols-3 gap-1">
					<button type="button" on:click={() => alignSelection('left')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Left</button>
					<button type="button" on:click={() => alignSelection('center')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Center</button>
					<button type="button" on:click={() => alignSelection('right')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Right</button>
					<button type="button" on:click={() => alignSelection('top')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Top</button>
					<button type="button" on:click={() => alignSelection('middle')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Middle</button>
					<button type="button" on:click={() => alignSelection('bottom')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Bottom</button>
				</div>
				<div class="grid grid-cols-2 gap-1">
					<button type="button" on:click={() => distributeSelection('horizontal')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Distribute H</button>
					<button type="button" on:click={() => distributeSelection('vertical')} class={`px-2 py-1 text-[11px] rounded border ${$theme === 'dark' ? 'border-stone-600 text-stone-200 hover:bg-stone-700' : 'border-stone-300 text-stone-700 hover:bg-stone-50'}`}>Distribute V</button>
				</div>
			</div>

			{#if showStrokeColors || hasFillableShapes}
				<div class={`grid gap-2 ${((showStrokeColors && hasFillableShapes) || compactFreehandStrokeRow) ? 'grid-cols-2 md:grid-cols-1' : 'grid-cols-1'}`}>
					{#if showStrokeColors}
						<div class="space-y-1.5 min-w-0">
							<fieldset class="flex flex-col gap-1.5 w-full min-w-0">
								<div class="flex items-center justify-between gap-2">
									<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke</legend>
									<button
										type="button"
										on:click={openStrokeColorPicker}
										class={`px-2 py-1 text-[10px] rounded border transition-colors ${$theme === 'dark' ? 'border-stone-600 text-stone-300 hover:bg-stone-700' : 'border-stone-300 text-stone-600 hover:bg-stone-100'}`}
										title="Fine tune custom stroke color"
									>
										Custom
									</button>
								</div>
								<div class="grid grid-cols-6 gap-1 md:gap-1.5 w-full">
									{#each strokeColors as color}
										<button
											type="button"
											on:click={() => updateStrokeColor(color)}
											class={`rounded-full border transition-all hover:scale-105 ${displayStrokeColor === color ? 'h-7 w-7 md:h-8 md:w-8 ring-2 ring-offset-1 ' + ($theme === 'dark' ? 'ring-stone-300 ring-offset-stone-800' : 'ring-stone-500 ring-offset-white') : 'h-6 w-6 md:h-7 md:w-7'} ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-300'}`}
											style="background-color: {color};"
											title={color}
										>
										</button>
									{/each}
								</div>
							</fieldset>
						</div>
					{/if}

					{#if compactFreehandStrokeRow}
						<div class="space-y-1.5 min-w-0">
							<fieldset class="space-y-1.5">
								<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke width</legend>
								<div class="flex items-center gap-1">
									<button
										on:click={() => updateStrokeWidthType(1)}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$defaultStrokeWidth === 1 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Thin"
										aria-label="Thin stroke"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
											<line x1="3" y1="12" x2="21" y2="12"></line>
										</svg>
									</button>
									<button
										on:click={() => updateStrokeWidthType(2)}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$defaultStrokeWidth === 2 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Medium"
										aria-label="Medium stroke"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<line x1="3" y1="12" x2="21" y2="12"></line>
										</svg>
									</button>
									<button
										on:click={() => updateStrokeWidthType(4)}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$defaultStrokeWidth === 4 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Thick"
										aria-label="Thick stroke"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
											<line x1="3" y1="12" x2="21" y2="12"></line>
										</svg>
									</button>
								</div>
							</fieldset>
						</div>
					{/if}

					{#if hasFillableShapes}
						<div class="space-y-1.5 min-w-0">
							<fieldset class="flex flex-col gap-1.5 w-full min-w-0">
								<div class="flex items-center justify-between gap-2">
									<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Background</legend>
									<button
										type="button"
										on:click={openFillColorPicker}
										class={`px-2 py-1 text-[10px] rounded border transition-colors ${$theme === 'dark' ? 'border-stone-600 text-stone-300 hover:bg-stone-700' : 'border-stone-300 text-stone-600 hover:bg-stone-100'}`}
										title="Fine tune custom fill color"
									>
										Custom
									</button>
								</div>
								<div class="grid grid-cols-6 gap-1 md:gap-1.5 w-full">
									<button
										type="button"
										on:click={() => updateFillColor(null)}
										class={`h-6 w-6 md:h-7 md:w-7 rounded-full border transition-all hover:scale-105 relative ${fillColor === null ? 'ring-2 ring-offset-1 ' + ($theme === 'dark' ? 'ring-stone-300 ring-offset-stone-800' : 'ring-stone-500 ring-offset-white') : ''} ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-300'}`}
										title="No fill"
									>
										<div class={`w-full h-full rounded-full absolute inset-0 ${$theme === 'dark' ? 'bg-stone-700' : 'bg-stone-200'}`} style="background-image: repeating-conic-gradient(${$theme === 'dark' ? '#374151' : '#e5e7eb'} 0% 25%, transparent 0% 50%); background-size: 50% 50%;"></div>
									</button>
									{#each fillColors as color}
										<button
											type="button"
											on:click={() => updateFillColor(color)}
											class={`rounded-full border transition-all hover:scale-105 ${fillColor === color ? 'h-7 w-7 md:h-8 md:w-8 ring-2 ring-offset-1 ' + ($theme === 'dark' ? 'ring-stone-300 ring-offset-stone-800' : 'ring-stone-500 ring-offset-white') : 'h-6 w-6 md:h-7 md:w-7'} ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-300'}`}
											style="background-color: {color};"
											title={color}
										>
										</button>
									{/each}
								</div>
							</fieldset>
						</div>
					{/if}
				</div>
			{/if}

			{#if false}
				<div class="flex items-center gap-2 min-w-0">
					<label for="line-width" class="sr-only">Line Width</label>
					<input
						type="range"
						min="0.5"
						max="20"
						step="0.5"
						bind:value={lineWidth}
						on:input={(e) => updateLineWidth(parseFloat((e.target as HTMLInputElement).value))}
						class={`flex-1 min-w-0 h-1 rounded-lg appearance-none cursor-pointer ${$theme === 'dark' ? 'bg-stone-600 accent-stone-400' : 'bg-stone-200 accent-stone-600'}`}
						id="line-width"
					/>
					<input
						type="number"
						bind:value={lineWidth}
						on:input={(e) => updateLineWidth(parseFloat((e.target as HTMLInputElement).value))}
						min="0.5"
						max="20"
						step="0.5"
						class={`w-12 px-1.5 py-1 text-xs border rounded focus:outline-none focus:ring-1 shrink-0 ${$theme === 'dark' ? 'border-stone-600 bg-stone-700 text-stone-200 focus:ring-stone-500' : 'border-stone-200 bg-stone-50 focus:ring-stone-400'}`}
						aria-label="Line width value"
					/>
				</div>
			{/if}

			{#if hasEditableEdges || (hasStrokeWidthControls && !compactFreehandStrokeRow)}
				<div class={`grid gap-2 ${hasEditableEdges && (hasStrokeWidthControls && !compactFreehandStrokeRow) ? 'grid-cols-2 md:grid-cols-1' : 'grid-cols-1'}`}>
					{#if hasEditableEdges}
						<div class="space-y-1.5 min-w-0">
							<fieldset class="space-y-1.5">
								<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Edges</legend>
								<div class="flex items-center gap-1">
									<button
										on:click={() => handleEdgeStyleChange('sharp')}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$edgeStyle === 'sharp' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Sharp"
										aria-label="Sharp edges"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
											<rect x="3" y="3" width="18" height="18" rx="0" ry="0" stroke-dasharray="2 2"></rect>
										</svg>
									</button>
									<button
										on:click={() => handleEdgeStyleChange('rounded')}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$edgeStyle === 'rounded' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Rounded"
										aria-label="Rounded edges"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
											<rect x="3" y="3" width="18" height="18" rx="2" ry="2" stroke-dasharray="2 2"></rect>
											<path d="M15 3h6v6" stroke-width="2.5" fill="none"></path>
										</svg>
									</button>
								</div>
							</fieldset>
						</div>
					{/if}

					{#if hasStrokeWidthControls && !compactFreehandStrokeRow}
						<div class="space-y-1.5 min-w-0">
							<fieldset class="space-y-1.5">
								<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke width</legend>
								<div class="flex items-center gap-1">
									<button
										on:click={() => updateStrokeWidthType(1)}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$defaultStrokeWidth === 1 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Thin"
										aria-label="Thin stroke"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
											<line x1="3" y1="12" x2="21" y2="12"></line>
										</svg>
									</button>
									<button
										on:click={() => updateStrokeWidthType(2)}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$defaultStrokeWidth === 2 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Medium"
										aria-label="Medium stroke"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<line x1="3" y1="12" x2="21" y2="12"></line>
										</svg>
									</button>
									<button
										on:click={() => updateStrokeWidthType(4)}
										class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$defaultStrokeWidth === 4 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
										title="Thick"
										aria-label="Thick stroke"
									>
										<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
											<line x1="3" y1="12" x2="21" y2="12"></line>
										</svg>
									</button>
								</div>
							</fieldset>
						</div>
					{/if}
				</div>
			{/if}

			{#if hasTexts}
				<div class="space-y-1.5">
					<fieldset class="flex flex-col gap-2 w-full min-w-0">
						<legend class={`text-xs font-medium mb-1 ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Text Align</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={() => updateTextAlign('left')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Left align"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="17" y1="10" x2="3" y2="10"></line>
									<line x1="21" y1="6" x2="3" y2="6"></line>
									<line x1="21" y1="14" x2="3" y2="14"></line>
									<line x1="17" y1="18" x2="3" y2="18"></line>
								</svg>
							</button>
							<button
								on:click={() => updateTextAlign('center')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Center align"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="18" y1="10" x2="6" y2="10"></line>
									<line x1="21" y1="6" x2="3" y2="6"></line>
									<line x1="21" y1="14" x2="3" y2="14"></line>
									<line x1="18" y1="18" x2="6" y2="18"></line>
								</svg>
							</button>
							<button
								on:click={() => updateTextAlign('right')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Right align"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="21" y1="10" x2="7" y2="10"></line>
									<line x1="21" y1="6" x2="3" y2="6"></line>
									<line x1="21" y1="14" x2="3" y2="14"></line>
									<line x1="21" y1="18" x2="7" y2="18"></line>
								</svg>
							</button>
						</div>
					</fieldset>
				</div>

				<div class="space-y-1.5">
					<fieldset class="flex flex-col gap-2 w-full min-w-0">
						<legend class={`text-xs font-medium mb-1 ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Opacity</legend>
						<div class="flex items-center gap-2 min-w-0">
							<input
								type="range"
								min="0"
								max="1"
								step="0.01"
								bind:value={textOpacity}
								on:input={(e) => updateTextOpacity(parseFloat((e.target as HTMLInputElement).value))}
								class={`flex-1 min-w-0 h-1 rounded-lg appearance-none cursor-pointer ${$theme === 'dark' ? 'bg-stone-600 accent-stone-400' : 'bg-stone-200 accent-stone-600'}`}
								aria-label="Text opacity"
							/>
							<input
								type="number"
								bind:value={textOpacity}
								on:input={(e) => updateTextOpacity(parseFloat((e.target as HTMLInputElement).value))}
								min="0"
								max="1"
								step="0.01"
								class={`w-16 px-1.5 py-1 text-xs border rounded focus:outline-none focus:ring-1 shrink-0 ${$theme === 'dark' ? 'border-stone-600 bg-stone-700 text-stone-200 focus:ring-stone-500' : 'border-stone-200 bg-stone-50 focus:ring-stone-400'}`}
								aria-label="Opacity value"
							/>
						</div>
					</fieldset>
				</div>
			{/if}

			{#if showDashPatternControls}
				<div class="grid grid-cols-2 gap-2 md:grid-cols-1">
					<div class="space-y-1.5 min-w-0">
						<fieldset class="space-y-1.5">
							<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke Style</legend>
							<div class="flex items-center gap-1">
								<button
									on:click={() => handleDashPatternChange('solid')}
									class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$dashPattern === 'solid' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
									title="Solid"
									aria-label="Solid line"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<line x1="3" y1="12" x2="21" y2="12"></line>
									</svg>
								</button>
								<button
									on:click={() => handleDashPatternChange('dashed')}
									class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$dashPattern === 'dashed' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
									title="Dashed"
									aria-label="Dashed line"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="4 4">
										<line x1="3" y1="12" x2="21" y2="12"></line>
									</svg>
								</button>
								<button
									on:click={() => handleDashPatternChange('dotted')}
									class={`flex flex-1 items-center justify-center p-1 md:p-1.5 rounded transition-colors ${$dashPattern === 'dotted' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
									title="Dotted"
									aria-label="Dotted line"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="2 2">
										<line x1="3" y1="12" x2="21" y2="12"></line>
									</svg>
								</button>
							</div>
						</fieldset>
					</div>

					<div class="space-y-1.5 min-w-0">
						<fieldset class="space-y-1.5">
							<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Layers</legend>
							<div class="flex items-center gap-1">
								<button
									on:click={sendToBack}
									class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
									title="Send to Back (Ctrl+Shift+[)"
									aria-label="Send to Back"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<line x1="12" y1="5" x2="12" y2="19"></line>
										<polyline points="19 12 12 19 5 12"></polyline>
									</svg>
								</button>
								<button
									on:click={sendBackward}
									class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
									title="Send Backward (Ctrl+[)"
									aria-label="Send Backward"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<polyline points="19 12 12 19 5 12"></polyline>
									</svg>
								</button>
								<button
									on:click={bringForward}
									class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
									title="Bring Forward (Ctrl+])"
									aria-label="Bring Forward"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<polyline points="5 12 12 5 19 12"></polyline>
									</svg>
								</button>
								<button
									on:click={bringToFront}
									class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
									title="Bring to Front (Ctrl+Shift+])"
									aria-label="Bring to Front"
								>
									<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<line x1="12" y1="19" x2="12" y2="5"></line>
										<polyline points="5 12 12 5 19 12"></polyline>
									</svg>
								</button>
							</div>
						</fieldset>
					</div>
				</div>
			{:else}
				<div class="space-y-1.5">
					<fieldset class="space-y-1.5">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Layers</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={sendToBack}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Send to Back (Ctrl+Shift+[)"
								aria-label="Send to Back"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="12" y1="5" x2="12" y2="19"></line>
									<polyline points="19 12 12 19 5 12"></polyline>
								</svg>
							</button>
							<button
								on:click={sendBackward}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Send Backward (Ctrl+[)"
								aria-label="Send Backward"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<polyline points="19 12 12 19 5 12"></polyline>
								</svg>
							</button>
							<button
								on:click={bringForward}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Bring Forward (Ctrl+])"
								aria-label="Bring Forward"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<polyline points="5 12 12 5 19 12"></polyline>
								</svg>
							</button>
							<button
								on:click={bringToFront}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Bring to Front (Ctrl+Shift+])"
								aria-label="Bring to Front"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="12" y1="19" x2="12" y2="5"></line>
									<polyline points="5 12 12 5 19 12"></polyline>
								</svg>
							</button>
						</div>
					</fieldset>
				</div>
			{/if}

			<div class="grid grid-cols-2 gap-2 md:grid-cols-1">
				<div class="space-y-1.5 min-w-0">
					<fieldset class="space-y-1.5">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Lock</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={handleLock}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${allLocked ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Lock"
								aria-label="Lock"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
									<path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
								</svg>
							</button>
							<button
								on:click={handleUnlock}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${!allLocked ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Unlock"
								aria-label="Unlock"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
									<path d="M7 11V7a5 5 0 0 1 9.9-1"></path>
								</svg>
							</button>
						</div>
					</fieldset>
				</div>

				<div class="space-y-1.5 min-w-0">
					<fieldset class="space-y-1.5">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Actions</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={handleDuplicate}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Duplicate (Ctrl+D)"
								aria-label="Duplicate"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
									<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
								</svg>
							</button>
							<button
								on:click={handleDelete}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Delete"
								aria-label="Delete"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<polyline points="3 6 5 6 21 6"></polyline>
									<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
									<line x1="10" y1="11" x2="10" y2="17"></line>
									<line x1="14" y1="11" x2="14" y2="17"></line>
								</svg>
							</button>
						</div>
					</fieldset>
				</div>
			</div>
		</div>
	</div>
{/if}
