// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

/**
 * The pole soft cap's geometry and curves (plan §7.11, P1-18). Plain Java, no Minecraft: the server
 * cap ({@link PoleCap}), Sable's wind, the client and the JUnit tests all read the same numbers.
 *
 * <p><b>Latitude</b> is the planet's projection's ({@link com.minecraftindustrialcomplex.deeptime.worlddata.Projection}: Lambert's
 * equal-area cylinder, φ = asin(−z / R), R = C/2π), so z ∈ [−R, R] spans pole to pole, north (−z) is
 * positive, and |z| = R is the pole edge. The cap starts at {@code capDeg} (φc, default 70°) and
 * becomes a hard stop at {@code stopDeg} (φs, default 80°); in blocks the band is R·(sin φs − sin φc):
 * 118 blocks at C = 16,384, 235 at 32,768, with 6.0 % of the planet behind the cap line (Ben,
 * 2026-10-03: from 75° the Lambert band would have been 49 blocks; on the old equirectangular map,
 * 75° to 80°, it was 228).
 *
 * <p><b>Depth</b> s = clamp((|φ| − φc) / (φs − φc), 0, 1): 0 at the cap line, 1 at the stop line.
 *
 * <p><b>The curves</b> (chosen in P1-18; the seam review page's curves were illustrations):
 * <ul>
 *   <li><b>Storm</b> (visual intensity, 0…1): s<sup>0.7</sup>. Concave, so the storm is already
 *       plain a little way in (0.2 at 71°, 0.62 at 75°) and a white-out at the stop.</li>
 *   <li><b>Pushback</b> (a velocity change toward the equator added every tick, blocks/tick²):
 *       {@link #PUSH_MAX} · s<sup>1.5</sup>, 0.10 at the stop. Convex, so the first degree is a
 *       breeze (0.009 at 72°) and the last one a gale. Vanilla ground friction (0.546 per tick)
 *       turns a per-tick push p into a steady drift of p / 0.454 ≈ 2.2 p, the same factor as the
 *       walking input (0.1): at 75° (p = 0.035) a walker keeps 65 % of his speed, at 78°
 *       (p = 0.072) 28 %, and at 80° walking is cancelled exactly while sprinting (0.13) still
 *       creeps. Anything airborne (drag 0.91, factor 11) or gliding (elytra, drag 0.99) is
 *       thrown back much harder, which is the point: no flying past the storm.</li>
 *   <li><b>Sable wind</b> (a velocity change toward the equator added every tick to a ship's
 *       rigid body, blocks/second per tick): {@link #WIND_MAX} · s<sup>1.5</sup>, i.e. up to
 *       20 blocks/s² at the stop (8 blocks/s² averaged over the band). On the Lambert map the band
 *       is 118 blocks (C = 16,384) or 235 (C = 32,768), so a 40 blocks/s jet that enters at 70°
 *       is turned round at about 79.4° (16k; the band as if s grew linearly in z) or 77° (32k); a
 *       slow airship much sooner.</li>
 *   <li><b>Hard stop</b>: poleward of φs nothing that the cap governs stays; it is put back on the
 *       stop line (players, vehicles) or has its poleward velocity cancelled and is blown back
 *       (Sable ships).</li>
 * </ul>
 *
 * @param circumference C in blocks
 * @param capDeg        φc, where the storm and the pushback start
 * @param stopDeg       φs, the hard stop
 */
public record PoleCapCurve(int circumference, double capDeg, double stopDeg) {

    /** ledger: POL-003. Ben, 2026-10-03: "Band from 70°, stop at 80°", so the band on the Lambert map
     *  is 118 blocks at C = 16,384 (49 from 75°). */
    public static final double DEFAULT_CAP_DEG = 70.0;
    public static final double DEFAULT_STOP_DEG = 80.0;

    /** Pushback at the stop line, blocks per tick per tick (see the class comment). */
    public static final double PUSH_MAX = 0.10;
    public static final double PUSH_EXPONENT = 1.5;
    public static final double STORM_EXPONENT = 0.7;
    /** Sable wind at the stop line, blocks per second added per tick (20 blocks/s²). */
    public static final double WIND_MAX = 1.0;

    /** Where a player or vehicle is relative to the cap. */
    public enum Zone { OPEN, STORM, STOP }

    public PoleCapCurve {
        if (circumference <= 0)
            throw new IllegalArgumentException("circumference must be positive, got " + circumference);
        if (!(capDeg > 0 && capDeg < stopDeg && stopDeg <= 90))
            throw new IllegalArgumentException("need 0 < pole_cap_deg < pole_stop_deg <= 90, got " + capDeg + " and " + stopDeg);
    }

    /** Latitude in degrees of block z, north (−z) positive (the planet's projection). */
    public static double latitudeDeg(int circumference, double z) {
        return com.minecraftindustrialcomplex.deeptime.worlddata.Projection.latitudeDeg(circumference, z);
    }

    /** The z of a latitude (north positive). */
    public static double zOfLatitude(int circumference, double latDeg) {
        return com.minecraftindustrialcomplex.deeptime.worlddata.Projection.zOfLatitude(circumference, latDeg);
    }

    /** Longitude in degrees of block x, folded to [−180°, 180°), east (+x) positive. */
    public static double longitudeDeg(int circumference, double x) {
        return com.minecraftindustrialcomplex.deeptime.worlddata.Projection.longitudeEastDeg(circumference, x);
    }

    public double latitudeDeg(double z) {
        return latitudeDeg(circumference, z);
    }

    /** |z| of the cap line (e.g. 2,450.3 at C = 16,384 for the default 70°). */
    public double capZ() {
        return Math.abs(zOfLatitude(circumference, capDeg));
    }

    /** |z| of the stop line (e.g. 2,568.0 at C = 16,384). */
    public double stopZ() {
        return Math.abs(zOfLatitude(circumference, stopDeg));
    }

    /** The depth s ∈ [0, 1] of block z into the cap band. */
    public double depth(double z) {
        double a = Math.abs(latitudeDeg(z));
        if (a <= capDeg)
            return 0.0;
        return Math.min(1.0, (a - capDeg) / (stopDeg - capDeg));
    }

    public Zone zone(double z) {
        double a = Math.abs(latitudeDeg(z));
        if (a > stopDeg)
            return Zone.STOP;
        return a > capDeg ? Zone.STORM : Zone.OPEN;
    }

    /** Storm intensity 0…1 (the client's fog and snow). */
    public double storm(double z) {
        double s = depth(z);
        return s == 0 ? 0 : Math.pow(s, STORM_EXPONENT);
    }

    /** Pushback magnitude, blocks/tick² toward the equator (0 up to the cap line). */
    public double push(double z) {
        double s = depth(z);
        return s == 0 ? 0 : PUSH_MAX * Math.pow(s, PUSH_EXPONENT);
    }

    /** Sable wind magnitude, blocks/second added per tick toward the equator. */
    public double wind(double z) {
        double s = depth(z);
        return s == 0 ? 0 : WIND_MAX * Math.pow(s, PUSH_EXPONENT);
    }

    /** +1 or −1: the sign of a z velocity pointing to the equator (0 exactly on it). */
    public static double equatorward(double z) {
        return z > 0 ? -1.0 : z < 0 ? 1.0 : 0.0;
    }

    /**
     * z moved back onto the stop line when it is past it, {@code inset} blocks on the equator side
     * (so the position is inside the stop, not on it); otherwise z unchanged.
     */
    public double clampZ(double z, double inset) {
        double limit = stopZ() - inset;
        if (z > limit)
            return limit;
        if (z < -limit)
            return -limit;
        return z;
    }

    /** True when |z| is past the stop line. */
    public boolean pastStop(double z) {
        return Math.abs(z) > stopZ();
    }
}
