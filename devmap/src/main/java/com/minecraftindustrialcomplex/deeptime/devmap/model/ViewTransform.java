// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

/**
 * The flat map's view: block (x, z) to screen (sx, sy) and back, north (−z) up and east (+x) right,
 * as Minecraft's own maps. The block (cx, cz) sits at the viewport's centre; {@code scale} is screen
 * pixels a block. x wraps with the world's period, so the centre is kept in [−period/2, period/2)
 * and the map is drawn once per period that the viewport reaches ({@link #firstCopy}, {@link #lastCopy}).
 */
public final class ViewTransform {

    private final double period;
    private double cx, cz, scale;
    private double left, top, width, height;

    public ViewTransform(double period, double cx, double cz, double scale) {
        this.period = period;
        this.cx = wrap(cx);
        this.cz = cz;
        this.scale = scale;
    }

    /** Sets the viewport, screen pixels. */
    public void viewport(double left, double top, double width, double height) {
        this.left = left;
        this.top = top;
        this.width = width;
        this.height = height;
    }

    public double cx() {
        return cx;
    }

    public double cz() {
        return cz;
    }

    public double scale() {
        return scale;
    }

    public double left() {
        return left;
    }

    public double top() {
        return top;
    }

    public double width() {
        return width;
    }

    public double height() {
        return height;
    }

    public double screenX(double x) {
        return left + width / 2.0 + (x - cx) * scale;
    }

    public double screenY(double z) {
        return top + height / 2.0 + (z - cz) * scale;
    }

    public double blockX(double sx) {
        return cx + (sx - left - width / 2.0) / scale;
    }

    public double blockZ(double sy) {
        return cz + (sy - top - height / 2.0) / scale;
    }

    /** Centres the view on block (x, z). */
    public void centre(double x, double z) {
        cx = wrap(x);
        cz = z;
    }

    /** Moves the view by a mouse drag of (dsx, dsy) screen pixels: the map follows the mouse. */
    public void pan(double dsx, double dsy) {
        cx = wrap(cx - dsx / scale);
        cz -= dsy / scale;
    }

    /** Clamps the centre's z into [zMin, zMax]. */
    public void clampZ(double zMin, double zMax) {
        cz = Math.max(zMin, Math.min(zMax, cz));
    }

    /** Zooms by {@code factor} about screen point (sx, sy): the block under it stays under it. */
    public void zoomAt(double sx, double sy, double factor, double minScale, double maxScale) {
        double bx = blockX(sx), bz = blockZ(sy);
        scale = Math.max(minScale, Math.min(maxScale, scale * factor));
        cx = wrap(bx - (sx - left - width / 2.0) / scale);
        cz = bz - (sy - top - height / 2.0) / scale;
    }

    public void scale(double s) {
        scale = s;
    }

    /** The lowest k for which the world copy shifted by k·period reaches the viewport. */
    public int firstCopy(double xMin, double xMax) {
        return (int) Math.floor((blockX(left) - xMax) / period);
    }

    /** The highest such k. */
    public int lastCopy(double xMin, double xMax) {
        return (int) Math.ceil((blockX(left + width) - xMin) / period);
    }

    private double wrap(double x) {
        double t = ((x + 0.5 * period) % period + period) % period;
        return (t >= period ? 0.0 : t) - 0.5 * period;
    }
}
