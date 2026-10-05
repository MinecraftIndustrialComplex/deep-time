// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import static com.minecraftindustrialcomplex.deeptime.worlddata.PortableMath.PI;

/**
 * Petroff's (2021) equal-area square map of a hemisphere, forward and inverse: the twin of
 * {@code deeptime_format::petroff} (whose documentation gives the construction and the source) and
 * of the pages' {@code DTPetroff}, operation for operation on {@link PortableMath}, so all three
 * give the same bits ({@code ProjectionParityTest}). Ported from the Rust original (LGPL-3.0-or-later),
 * itself written from the paper's formulas (NOTICE.md).
 */
public final class Petroff {

    private final double sinPhi0, cosPhi0, hp, xip, psi0, psi1, p0p, p1p, p2p, cA, cB, f0, f1, a1p, c01p, c2p, b1, b2;
    private final double[] tanB;

    public Petroff() {
        double phi0 = 3.0 * PI / 8.0;
        sinPhi0 = PortableMath.sin(phi0);
        cosPhi0 = PortableMath.cos(phi0);
        double a1 = PortableMath.asin(1.0 / Math.sqrt(2.0 - cosPhi0 * cosPhi0));
        double a2 = PortableMath.asin(2.0 * sinPhi0 / Math.sqrt(3.0 - PortableMath.cos(2.0 * phi0)));
        hp = 12.0 / PI * (a1 + a2 - PI / 2.0);
        double s3 = Math.sqrt(3.0);
        xip = PortableMath.atan((PI * (hp - 3.0) * (hp - 3.0))
                / (s3 * (PI * (hp * hp - 2.0 * hp + 45.0) - 96.0 * a1 - 48.0 * a2)));
        psi0 = a1;
        psi1 = PI - 2.0 * psi0;
        p0p = PortableMath.atan(s3 / hp);
        p1p = 7.0 * PI / 6.0 - p0p - xip;
        p2p = xip - PI / 6.0;
        cA = PortableMath.acos(cosPhi0 / Math.sqrt(2.0));
        cB = PI / 2.0 - phi0;
        a1p = Math.sqrt(hp * hp + 3.0) * PortableMath.sin(PI / 3.0 - PortableMath.atan(hp / s3)) / PortableMath.sin(xip);
        b1 = PortableMath.atan(Math.sqrt(2.0) * PortableMath.tan(phi0));
        b2 = PI / 2.0 - b1;
        f0 = a2;
        f1 = PI / 2.0 - a2;
        c01p = Math.sqrt(hp * hp + 3.0);
        c2p = 3.0 - hp;
        tanB = new double[]{PortableMath.tan(PI / 4.0), PortableMath.tan(b1), PortableMath.tan(b2)};
    }

    private static double sign(double x) {
        return x > 0.0 ? 1.0 : (x < 0.0 ? -1.0 : 0.0);
    }

    private static double sqrt0(double x) {
        return x > 0.0 ? Math.sqrt(x) : 0.0;
    }

    /** The square point (x, y) of (λ, φ) in radians (polar aspect, northern hemisphere for φ ≥ 0). */
    public double[] forward(double lambda, double phi) {
        double twoPi = 2.0 * PI;
        double lam = lambda - twoPi * Math.floor(lambda / twoPi);
        if (lam >= twoPi || lam < 0.0)
            lam = 0.0;
        double phiC = Math.abs(phi);
        double lambdaC = lam - PI / 4.0;
        double quadrant = Math.floor(2.0 * lam / PI);
        if (quadrant > 3.0)
            quadrant = 3.0;
        double lambda0 = quadrant * PI / 2.0;
        double dl = lambdaC - lambda0;
        double sinPc = PortableMath.sin(phiC), cosPc = PortableMath.cos(phiC);
        double sinDl = PortableMath.sin(dl), cosDl = PortableMath.cos(dl);
        double theta = Math.abs(PortableMath.atan2(cosPc * sinDl, sinPhi0 * cosPc * cosDl - cosPhi0 * sinPc));
        double r = PortableMath.acos(sinPhi0 * sinPc + cosPhi0 * cosPc * cosDl);
        int s = theta <= psi0 ? 0 : (theta <= psi0 + psi1 ? 1 : 2);
        double beta = s == 0 ? psi0 - theta : (s == 1 ? theta - psi0 : PI - theta);
        double c = s <= 1 ? cA : cB;
        double g = s == 1 ? psi1 : psi0;
        double gp = s == 0 ? p0p : (s == 1 ? p1p : p2p);
        double f = s == 0 ? f0 : (s == 1 ? f1 : PI / 4.0);
        double ap = s == 0 ? hp : a1p;
        double cp = s <= 1 ? c01p : c2p;

        double sinR = PortableMath.sin(r), cosR = PortableMath.cos(r);
        double sinC = PortableMath.sin(c), cosC = PortableMath.cos(c);
        double x = PortableMath.acos(cosR * cosC + sinR * sinC * PortableMath.cos(beta));
        double gamma = x > 0.0 ? PortableMath.asin(PortableMath.sin(beta) * sinR / PortableMath.sin(x)) : 0.0;
        double sinG = PortableMath.sin(g), cosG = PortableMath.cos(g);
        double epsilon = PortableMath.acos(sinG * PortableMath.sin(gamma) * cosC - cosG * PortableMath.cos(gamma));
        double upupvp = (gamma + g + epsilon - PI) / (f + g - PI / 2.0);
        double sinEps = PortableMath.sin(epsilon);
        double cosXy;
        if (sinEps != 0.0) {
            double q = sinG * sinC / sinEps;
            cosXy = sqrt0(1.0 - q * q);
        } else {
            cosXy = 0.0;
        }
        double den = 1.0 - cosXy;
        double xpxpyp = den > 0.0 ? sqrt0((1.0 - PortableMath.cos(x)) / den) : 0.0;
        double up = ap * upupvp;
        double xpyp = sqrt0(up * up + cp * cp - 2.0 * up * cp * PortableMath.cos(gp));
        double cosGammap;
        if (xpyp > 0.0) {
            double q = up * PortableMath.sin(gp) / xpyp;
            cosGammap = sqrt0(1.0 - q * q);
        } else {
            cosGammap = 1.0;
        }
        double xp = xpyp * xpxpyp;
        double yp = xpyp - xp;
        double rp = sqrt0(xp * xp + cp * cp - 2.0 * xp * cp * cosGammap);
        double alphap;
        if (up * rp <= 0.0) {
            alphap = xip - PI / 6.0;
        } else {
            double q = (yp * yp - up * up - rp * rp) / (-2.0 * up * rp);
            alphap = q < 1.0 ? PortableMath.acos(q) : 0.0;
        }
        double thetap = s == 0 ? alphap
                : (s == 1 ? PI - (xip - PI / 6.0) - alphap : PI - (xip - PI / 6.0) + alphap);
        double side = dl < 0.0 ? -1.0 : 1.0;
        double xC = side * rp * PortableMath.sin(thetap);
        double yC = hp - rp * PortableMath.cos(thetap);
        double yH = yC * sign(phi) - 3.0;
        double zeta = PI / 4.0 + quadrant * PI / 2.0;
        double sinZ = PortableMath.sin(zeta), cosZ = PortableMath.cos(zeta);
        double s3 = Math.sqrt(3.0);
        double k = s3 / (3.0 * Math.sqrt(2.0));
        return new double[]{(xC * cosZ - yH * sinZ / s3) * k, (xC * sinZ + yH * cosZ / s3) * k};
    }

    /** (λ, φ) in radians of the square point (x, y). */
    public double[] inverse(double xM, double yM) {
        double quadrant = (xM <= 0.0 && yM >= 0.0) ? 2.0 : (yM >= 0.0 ? 1.0 : (xM >= 0.0 ? 0.0 : 3.0));
        double zeta = PI / 4.0 + quadrant * PI / 2.0;
        double sinZ = PortableMath.sin(zeta), cosZ = PortableMath.cos(zeta);
        double xC = Math.sqrt(6.0) * (xM * cosZ + yM * sinZ);
        double yC = 3.0 * Math.sqrt(2.0) * (yM * cosZ - xM * sinZ);
        double sgnphi, yH;
        if (yC < -3.0) {
            sgnphi = -1.0;
            yH = -6.0 - yC;
        } else {
            sgnphi = 1.0;
            yH = yC;
        }
        double dy = hp - 3.0 - yH;
        double rp = Math.sqrt(xC * xC + dy * dy);
        double thetap = Math.abs(PortableMath.atan2(xC, dy));
        int s = thetap <= p0p ? 0 : (thetap <= p0p + p1p ? 1 : 2);
        double alphap = s == 0 ? thetap : (s == 1 ? PI - p2p - thetap : thetap + p2p - PI);
        double c = s <= 1 ? cA : cB;
        double g = s == 1 ? psi1 : psi0;
        double gp = s == 0 ? p0p : (s == 1 ? p1p : p2p);
        double f = s == 0 ? f0 : (s == 1 ? f1 : PI / 4.0);
        double ap = s == 0 ? hp : a1p;
        double cp = s <= 1 ? c01p : c2p;
        double b = s == 0 ? PI / 4.0 : (s == 1 ? b1 : b2);
        double tb = tanB[s];

        double betap = gp - alphap;
        double xp = sqrt0(rp * rp + cp * cp - 2.0 * rp * cp * PortableMath.cos(betap));
        double gammap = xp > 0.0 ? PortableMath.asin(rp * PortableMath.sin(betap) / xp) : 0.0;
        double epsp = PI - gp - gammap;
        double yp = epsp > 0.0 ? rp * PortableMath.sin(alphap) / PortableMath.sin(epsp) : 0.0;
        double xy = xp + yp;
        double up = Math.sqrt(Math.abs(cp * cp + xy * xy - 2.0 * cp * xy * PortableMath.cos(gammap)));
        double vp = ap - up;
        double w = vp * (f + g - PI / 2.0) / ap;
        double num = -PortableMath.sin(w);
        double den = PortableMath.cos(b) - PortableMath.cos(w);
        double delta = (den == 0.0 && num == 0.0) ? 0.0 : PortableMath.atan(num / den);
        double gamma = f - delta;
        double tq = tb / PortableMath.cos(delta);
        double cosXy = 1.0 / Math.sqrt(1.0 + tq * tq);
        double x;
        if (xy > 0.0) {
            double q = xp / xy;
            x = PortableMath.acos(1.0 - q * q * (1.0 - cosXy));
        } else {
            x = 0.0;
        }
        double sinX = PortableMath.sin(x), cosX = PortableMath.cos(x);
        double sinC = PortableMath.sin(c), cosC = PortableMath.cos(c);
        double r = PortableMath.acos(cosX * cosC + sinX * sinC * PortableMath.cos(gamma));
        double sinR = PortableMath.sin(r);
        double beta = sinR != 0.0 ? PortableMath.asin(sinX * PortableMath.sin(gamma) / sinR) : 0.0;
        double alpha = s == 0 ? psi0 - beta : (s == 1 ? psi0 + beta : PI - beta);
        double cosR = PortableMath.cos(r);
        double phiH = PortableMath.asin(sinPhi0 * cosR - cosPhi0 * sinR * PortableMath.cos(alpha));
        double lambda0 = PI / 4.0 + quadrant * PI / 2.0;
        double phi = sgnphi * phiH;
        num = PortableMath.sin(alpha) * sinR * cosPhi0;
        den = cosR - sinPhi0 * PortableMath.sin(phiH);
        double dl = (den == 0.0 && num == 0.0) ? 0.0 : PortableMath.atan(num / den);
        double lambda = lambda0 + sign(xC) * dl;
        if (Math.abs(phi) == PI / 2.0)
            lambda = 0.0;
        return new double[]{lambda, phi};
    }
}
