import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, diamonds, images, paths, selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedImages, selectedPaths, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond, type Image, type Path } from '$lib/stores/editor';
import { updatePaths } from '$lib/utils/canvas-operations/path';
import type { ClipboardData } from './clipboard';

function calculateBoundingBox(clipboard: ClipboardData, fallbackX: number, fallbackY: number): { minX: number; minY: number } {
    const minXValues: number[] = [];
    const minYValues: number[] = [];

    clipboard.rectangles.forEach(r => {
        minXValues.push(r.position.x);
        minYValues.push(r.position.y);
    });

    clipboard.ellipses.forEach(e => {
        minXValues.push(e.position.x - e.radius_x);
        minYValues.push(e.position.y - e.radius_y);
    });

    clipboard.diamonds.forEach(d => {
        minXValues.push(d.position.x);
        minYValues.push(d.position.y);
    });

    clipboard.lines.forEach(l => {
        minXValues.push(Math.min(l.start.x, l.end.x));
        minYValues.push(Math.min(l.start.y, l.end.y));
    });

    clipboard.arrows.forEach(a => {
        minXValues.push(Math.min(a.start.x, a.end.x));
        minYValues.push(Math.min(a.start.y, a.end.y));
    });

    clipboard.images.forEach(i => {
        minXValues.push(i.position.x);
        minYValues.push(i.position.y);
    });

    clipboard.paths.forEach(p => {
        if (p.points.length > 0) {
            const minX = Math.min(...p.points.map(pt => pt.x));
            const minY = Math.min(...p.points.map(pt => pt.y));
            minXValues.push(minX);
            minYValues.push(minY);
        }
    });

    if (minXValues.length === 0) return { minX: fallbackX, minY: fallbackY };
    return {
        minX: Math.min(...minXValues),
        minY: Math.min(...minYValues)
    };
}

export function pasteShapes(clipboard: ClipboardData, offsetX: number, offsetY: number): {
    rectangles: number[];
    ellipses: number[];
    lines: number[];
    arrows: number[];
    diamonds: number[];
    images: number[];
    paths: number[];
} {
    const api = get(editorApi);
    if (!api) return { rectangles: [], ellipses: [], lines: [], arrows: [], diamonds: [], images: [], paths: [] };

    const { minX, minY } = calculateBoundingBox(clipboard, offsetX, offsetY);

    api.save_snapshot();

    const pastedIds = {
        rectangles: [] as number[],
        ellipses: [] as number[],
        lines: [] as number[],
        arrows: [] as number[],
        diamonds: [] as number[],
        images: [] as number[],
        paths: [] as number[]
    };

    clipboard.rectangles.forEach(rect => {
        const newX = rect.position.x - minX + offsetX;
        const newY = rect.position.y - minY + offsetY;
        const newId = api.add_rectangle_without_snapshot(newX, newY, rect.width, rect.height);
        if (rect.stroke_color) {
            api.set_rectangle_stroke_color(BigInt(newId), rect.stroke_color, false);
        }
        if (rect.fill_color !== undefined && rect.fill_color !== null) {
            api.set_rectangle_fill_color(BigInt(newId), rect.fill_color, false);
        }
        if (rect.line_width) {
            api.set_rectangle_line_width(BigInt(newId), rect.line_width, false);
        }
        if (rect.dash_pattern) {
            api.set_rectangle_dash_pattern(BigInt(newId), rect.dash_pattern, false);
        }
        if (rect.border_radius !== undefined) {
            api.set_rectangle_border_radius(BigInt(newId), rect.border_radius, false);
        }
        if (rect.rotation_angle !== undefined) {
            api.set_rectangle_rotation(BigInt(newId), rect.rotation_angle, false);
        }
        pastedIds.rectangles.push(Number(newId));
    });

    clipboard.ellipses.forEach(ellipse => {
        const newX = ellipse.position.x - minX + offsetX;
        const newY = ellipse.position.y - minY + offsetY;
        const newId = api.add_ellipse_without_snapshot(newX, newY, ellipse.radius_x, ellipse.radius_y);
        if (ellipse.stroke_color) {
            api.set_ellipse_stroke_color(BigInt(newId), ellipse.stroke_color, false);
        }
        if (ellipse.fill_color !== undefined && ellipse.fill_color !== null) {
            api.set_ellipse_fill_color(BigInt(newId), ellipse.fill_color, false);
        }
        if (ellipse.line_width) {
            api.set_ellipse_line_width(BigInt(newId), ellipse.line_width, false);
        }
        if (ellipse.dash_pattern) {
            api.set_ellipse_dash_pattern(BigInt(newId), ellipse.dash_pattern, false);
        }
        if (ellipse.rotation_angle !== undefined) {
            api.set_ellipse_rotation(BigInt(newId), ellipse.rotation_angle, false);
        }
        pastedIds.ellipses.push(Number(newId));
    });

    clipboard.diamonds.forEach(diamond => {
        const newX = diamond.position.x - minX + offsetX;
        const newY = diamond.position.y - minY + offsetY;
        const newId = api.add_diamond_without_snapshot(newX, newY, diamond.width, diamond.height);
        if (diamond.stroke_color) {
            api.set_diamond_stroke_color(BigInt(newId), diamond.stroke_color, false);
        }
        if (diamond.fill_color !== undefined && diamond.fill_color !== null) {
            api.set_diamond_fill_color(BigInt(newId), diamond.fill_color, false);
        }
        if (diamond.line_width) {
            api.set_diamond_line_width(BigInt(newId), diamond.line_width, false);
        }
        if (diamond.dash_pattern) {
            api.set_diamond_dash_pattern(BigInt(newId), diamond.dash_pattern, false);
        }
        if (diamond.border_radius !== undefined) {
            api.set_diamond_border_radius(BigInt(newId), diamond.border_radius, false);
        }
        if (diamond.rotation_angle !== undefined) {
            api.set_diamond_rotation(BigInt(newId), diamond.rotation_angle, false);
        }
        pastedIds.diamonds.push(Number(newId));
    });

    clipboard.lines.forEach(line => {
        const startX = line.start.x - minX + offsetX;
        const startY = line.start.y - minY + offsetY;
        const endX = line.end.x - minX + offsetX;
        const endY = line.end.y - minY + offsetY;
        const newId = api.add_line_without_snapshot(startX, startY, endX, endY);
        if (line.stroke_color) {
            api.set_line_stroke_color(BigInt(newId), line.stroke_color, false);
        }
        if (line.line_width) {
            api.set_line_line_width(BigInt(newId), line.line_width, false);
        }
        if (line.dash_pattern) {
            api.set_line_dash_pattern(BigInt(newId), line.dash_pattern, false);
        }
        if (line.rotation_angle !== undefined) {
            api.set_line_rotation(BigInt(newId), line.rotation_angle, false);
        }
        pastedIds.lines.push(Number(newId));
    });

    clipboard.arrows.forEach(arrow => {
        const startX = arrow.start.x - minX + offsetX;
        const startY = arrow.start.y - minY + offsetY;
        const endX = arrow.end.x - minX + offsetX;
        const endY = arrow.end.y - minY + offsetY;
        const newId = api.add_arrow_without_snapshot(startX, startY, endX, endY);
        if (arrow.stroke_color) {
            api.set_arrow_stroke_color(BigInt(newId), arrow.stroke_color, false);
        }
        if (arrow.line_width) {
            api.set_arrow_line_width(BigInt(newId), arrow.line_width, false);
        }
        if (arrow.dash_pattern) {
            api.set_arrow_dash_pattern(BigInt(newId), arrow.dash_pattern, false);
        }
        if (arrow.rotation_angle !== undefined) {
            api.set_arrow_rotation(BigInt(newId), arrow.rotation_angle, false);
        }
        pastedIds.arrows.push(Number(newId));
    });

    clipboard.images.forEach(image => {
        const newX = image.position.x - minX + offsetX;
        const newY = image.position.y - minY + offsetY;
        const newId = api.add_image_without_snapshot(newX, newY, image.width, image.height, image.image_data);
        if (image.rotation_angle !== undefined) {
            api.set_image_rotation(BigInt(newId), image.rotation_angle, false);
        }
        pastedIds.images.push(Number(newId));
    });

    clipboard.paths.forEach(path => {
        if (path.points.length > 0) {
            const offsetPoints = path.points.map(pt => ({
                x: pt.x - minX + offsetX,
                y: pt.y - minY + offsetY
            }));
            const newId = api.add_path_without_snapshot(offsetPoints);
            if (path.stroke_color) {
                api.set_path_stroke_color(BigInt(newId), path.stroke_color, false);
            }
            if (path.line_width) {
                api.set_path_line_width(BigInt(newId), path.line_width, false);
            }
            if (path.rotation_angle !== undefined) {
                api.set_path_rotation(BigInt(newId), path.rotation_angle, false);
            }
            pastedIds.paths.push(Number(newId));
        }
    });

    api.save_snapshot();

    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    const updatedEllipses = Array.from(api.get_ellipses() as Ellipse[]);
    const updatedLines = Array.from(api.get_lines() as Line[]);
    const updatedArrows = Array.from(api.get_arrows() as Arrow[]);
    const updatedDiamonds = Array.from(api.get_diamonds() as Diamond[]);
    const updatedImages = Array.from(api.get_images() as Image[]);

    rectangles.set(updatedRectangles);
    ellipses.set(updatedEllipses);
    lines.set(updatedLines);
    arrows.set(updatedArrows);
    diamonds.set(updatedDiamonds);
    images.set(updatedImages);

    updatePaths();
    const updatedPaths = Array.from(api.get_paths() as Path[]);
    paths.set(updatedPaths);

    selectedRectangles.set(updatedRectangles.filter(r => pastedIds.rectangles.includes(r.id)));
    selectedEllipses.set(updatedEllipses.filter(e => pastedIds.ellipses.includes(e.id)));
    selectedLines.set(updatedLines.filter(l => pastedIds.lines.includes(l.id)));
    selectedArrows.set(updatedArrows.filter(a => pastedIds.arrows.includes(a.id)));
    selectedDiamonds.set(updatedDiamonds.filter(d => pastedIds.diamonds.includes(d.id)));
    selectedImages.set(updatedImages.filter(i => pastedIds.images.includes(i.id)));
    selectedPaths.set(updatedPaths.filter(p => pastedIds.paths.includes(p.id)));

    return pastedIds;
}

