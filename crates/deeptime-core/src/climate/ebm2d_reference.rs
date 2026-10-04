// SPDX-License-Identifier: LGPL-3.0-or-later
//! The per-cell energy balance's year loop as it was before P1-17 factored its implicit solves
//! once per run and dropped the per-step allocations: kept verbatim for the tests, which compare
//! the two bit for bit.

use super::*;

pub fn run(
    p: &Params2d,
    g: &Grid,
    s: &Surface,
    state: &mut State2d,
    ocean: Option<&dyn OceanTransport>,
    years: u32,
    itcz0: [f64; 12],
) -> Result2d {
    let ep = &p.ebm;
    let (w, h) = (g.w, g.h);
    let n = w * h;
    if !state.fits(n) {
        *state = State2d::from_bands(g, &EbmState::default());
    }
    let spy = ep.steps_per_year.max(12);
    let per_month = spy / 12;
    let dt = YEAR_S / spy as f64;
    let lat: Vec<f64> = (0..h).map(|j| g.lat_c(j)).collect();
    let area: Vec<f64> = g.areas();
    let area_tot: f64 = area.iter().sum();
    let r2 = p.radius_m * p.radius_m;
    let f: Vec<f64> = (0..n)
        .map(|c| s.land.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0))
        .collect();
    let sheet: Vec<f64> = (0..n)
        .map(|c| s.sheet.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0))
        .collect();
    let desert: Vec<f64> = (0..n)
        .map(|c| s.desert.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0))
        .collect();
    let p2 = |l: f64| {
        let x = l.to_radians().sin();
        0.5 * (3.0 * x * x - 1.0)
    };
    let free_sea: Vec<f64> = lat.iter().map(|&l| ep.a0 + ep.a2 * p2(l)).collect();
    let free_land: Vec<f64> = (0..n)
        .map(|c| free_sea[c / w] + p.desert_albedo * desert[c])
        .collect();
    let frozen = |t: f64| 1.0 - smoothstep(ep.ice_temp_c - 3.0, ep.ice_temp_c + 3.0, t);
    // Insolation of every step and row (the step's midpoint).
    let q: Vec<Vec<f64>> = (0..spy)
        .map(|st| {
            let lam = ep.orbit.lambda_at((st as f64 + 0.5) / spy as f64);
            lat.iter()
                .map(|&l| {
                    insolation::daily_orbit(ep.s0, l.to_radians(), lam, ep.obliquity_rad, &ep.orbit)
                })
                .collect()
        })
        .collect();
    // Conductances (W/K per unit sphere area per R², i.e. D·G) of every face, the atmosphere's
    // and the slab's. East–west face (i, j)–(i + 1, j); north–south face (i, j)–(i, j + 1).
    let k_ew: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            let c1 = j * w + (i + 1) % w;
            p.zonal_ratio * p.d_face(lat[j], 0.5 * (f[c] + f[c1])) * g.g_ew(i, j)
        })
        .collect();
    let k_ns: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            if j + 1 >= h {
                return 0.0;
            }
            let c1 = c + w;
            p.d_face(g.lat_edge[j + 1], 0.5 * (f[c] + f[c1])) * g.g_ns(i, j)
        })
        .collect();
    let sea_min = |a: usize, b: usize| (1.0 - f[a]).min(1.0 - f[b]);
    let ko_ew: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            ep.d_ocean * sea_min(c, j * w + (i + 1) % w) * g.g_ew(i, j)
        })
        .collect();
    let ko_ns: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            if j + 1 >= h {
                0.0
            } else {
                ep.d_ocean * sea_min(c, c + w) * g.g_ns(i, j)
            }
        })
        .collect();
    // Capacities per cell, J/K per unit area (× A): the atmosphere's mean T̄ has C_l; the slab's
    // sea share C_m (1 − f).
    let cap_atm: Vec<f64> = area.iter().map(|a| ep.c_land * a).collect();
    let cap_sea: Vec<f64> = (0..n)
        .map(|c| ep.c_ocean * area[c] * (1.0 - f[c]).max(1e-6))
        .collect();
    // The zonal shift's cell width of every row, m.
    let dx_row: Vec<f64> = (0..h)
        .map(|j| p.radius_m * g.cos_mean(j).max(1e-6) * g.lon_w[0])
        .collect();

    let mut out = Result2d {
        heat_atm_pw: vec![0.0; h.saturating_sub(1)],
        heat_ocean_diff_pw: vec![0.0; h.saturating_sub(1)],
        heat_ocean_pw: vec![0.0; h.saturating_sub(1)],
        itcz: itcz0,
        ..Result2d::default()
    };
    let mut itcz = itcz0;
    let mut prev: Option<(Vec<f64>, Vec<f64>)> = None;
    for year in 0..years.max(1) {
        let mut ml = vec![[0.0; 12]; n];
        let mut mo = vec![[0.0; 12]; n];
        let mut mm = vec![[0.0; 12]; n];
        let mut mi = vec![[0.0; 12]; n];
        let mut row_t = vec![[0.0f64; 12]; h];
        let (mut asr, mut olr, mut tg) = (0.0, 0.0, 0.0);
        let mut north_atm = vec![0.0; h.saturating_sub(1)];
        let mut north_od = vec![0.0; h.saturating_sub(1)];
        let mut north_o = vec![0.0; h.saturating_sub(1)];
        for (st, qs) in q.iter().enumerate() {
            let month = st / per_month;
            // 1. Local terms, implicit (`ebm.rs`'s 3 × 3 solve), cells in parallel; each returns
            //    its new tiles and its budget.
            let local: Vec<(f64, f64, f64, f64, f64, f64)> = (0..n)
                .into_par_iter()
                .map(|c| {
                    let j = c / w;
                    let (to, tl, tm) = (state.t_ocean[c], state.t_land[c], state.t_mixed[c]);
                    let mut ice = state.ice_j_m2[c];
                    let h_ice = ice / ICE_J_M3;
                    let cover = (h_ice / FULL_ICE_M).min(1.0);
                    let insulated = smoothstep(0.5 * SEA_ICE_CAP_M, SEA_ICE_CAP_M, h_ice);
                    let gc = OPEN_WATER_W_M2_K * (1.0 - cover)
                        + (ICE_W_M_K / h_ice.max(0.05)).min(OPEN_WATER_W_M2_K)
                            * cover
                            * (1.0 - insulated);
                    let ao = free_sea[j] + (ep.ai - free_sea[j]) * frozen(to).max(cover);
                    let al = free_land[c] + (ep.ai - free_land[c]) * frozen(tl).max(sheet[c]);
                    let so = qs[j] * (1.0 - ao);
                    let sl = qs[j] * (1.0 - al);
                    let (cl, cs, cm) = (ep.c_land / dt, ep.c_land / dt, ep.c_ocean / dt);
                    let (xl, xo) = (ep.exchange * (1.0 - f[c]), ep.exchange * f[c]);
                    let (a_l, b_l) = crate::climate::co2::longwave(
                        ep.a,
                        ep.b,
                        ep.co2.as_ref(),
                        tl,
                        ep.snow_cloud,
                        frozen(tl).max(sheet[c]),
                    );
                    let (a_o, b_o) = crate::climate::co2::longwave(
                        ep.a,
                        ep.b,
                        ep.co2.as_ref(),
                        to,
                        ep.snow_cloud,
                        cover,
                    );
                    let (r1, r2, r3) = (cl * tl + sl - a_l, cs * to + so - a_o, cm * tm);
                    let a33 = cm + gc;
                    let a11 = cl + b_l + xl;
                    let a22 = cs + b_o + xo + gc - gc * gc / a33;
                    let r2 = r2 + gc * r3 / a33;
                    let det = a11 * a22 - xl * xo;
                    let tl2 = (r1 * a22 + xl * r2) / det;
                    let mut to2 = (a11 * r2 + xo * r1) / det;
                    let mut tm2 = (r3 + gc * to2) / a33;
                    let a_sr = f[c] * sl + (1.0 - f[c]) * so;
                    let a_lr = f[c] * (a_l + b_l * tl2) + (1.0 - f[c]) * (a_o + b_o * to2);
                    if ice > 0.0 && to2 > 0.0 {
                        let melt = (to2 * ep.c_land).min(ice);
                        ice -= melt;
                        to2 -= melt / ep.c_land;
                    }
                    // At once, as `ebm.rs` does: the slab's diffusion then never sees a slab
                    // below the freezing point.
                    freeze_melt(&mut tm2, &mut ice, ep.c_ocean);
                    (tl2, to2, tm2, ice, a_sr, a_lr)
                })
                .collect();
            for (c, &(tl, to, tm, ice, a_sr, a_lr)) in local.iter().enumerate() {
                state.t_land[c] = tl;
                state.t_ocean[c] = to;
                state.t_mixed[c] = tm;
                state.ice_j_m2[c] = ice;
                asr += area[c] * a_sr;
                olr += area[c] * a_lr;
            }
            // 2. The atmosphere's diffusion of T̄, implicit ADI: rows (cyclic), then columns.
            let mean: Vec<f64> = (0..n)
                .map(|c| f[c] * state.t_land[c] + (1.0 - f[c]) * state.t_ocean[c])
                .collect();
            let mut tb = diffuse_x(w, h, &mean, &k_ew, &cap_atm, dt);
            tb = diffuse_y(w, h, &tb, &k_ns, &cap_atm, dt);
            northward(w, &k_ns, &tb, r2 * dt, &mut north_atm);
            // 3. The prevailing wind's zonal shift of T̄.
            if p.wind_heat > 0.0 {
                let hemi = p.hemi_shift[month];
                tb.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
                    let phi = lat[j];
                    let pb = phi - belts::shift(phi, itcz[month], hemi);
                    let u = p.wind_heat * low_level_wind(pb);
                    shift_row(row, u * dt / dx_row[j]);
                });
            }
            for c in 0..n {
                let d = tb[c] - mean[c];
                state.t_land[c] += d;
                state.t_ocean[c] += d;
            }
            // 4. The ocean: the slab's diffusion through the sea, then its transport.
            if ep.d_ocean > 0.0 {
                let mut tm = diffuse_x(w, h, &state.t_mixed, &ko_ew, &cap_sea, dt);
                tm = diffuse_y(w, h, &tm, &ko_ns, &cap_sea, dt);
                northward(w, &ko_ns, &tm, r2 * dt, &mut north_od);
                state.t_mixed = tm;
            }
            if let Some(o) = ocean {
                o.step(&mut state.t_mixed, dt, &mut north_o);
            }
            // 5. Freezing and melting at the slab.
            for c in 0..n {
                freeze_melt(&mut state.t_mixed[c], &mut state.ice_j_m2[c], ep.c_ocean);
            }
            let pm = per_month as f64;
            for c in 0..n {
                let (to, tl) = (state.t_ocean[c], state.t_land[c]);
                ml[c][month] += tl / pm;
                mo[c][month] += to / pm;
                mm[c][month] += state.t_mixed[c] / pm;
                mi[c][month] += (state.ice_j_m2[c] / ICE_J_M3 / FULL_ICE_M).min(1.0) / pm;
                let tbar = f[c] * tl + (1.0 - f[c]) * to;
                tg += area[c] * tbar;
                row_t[c / w][month] += tbar / pm;
            }
        }
        let steps = spy as f64 * area_tot;
        out.asr = asr / steps;
        out.olr = olr / steps;
        out.t_global = tg / steps;
        out.t_land = ml;
        out.t_ocean = mo;
        out.t_mixed = mm;
        out.sea_ice = mi;
        out.years = year + 1;
        let pw = 1.0e15 * YEAR_S;
        out.heat_atm_pw = north_atm.iter().map(|x| x / pw).collect();
        out.heat_ocean_diff_pw = north_od.iter().map(|x| x / pw).collect();
        out.heat_ocean_pw = north_o.iter().map(|x| x / pw).collect();
        // The next year's ITCZ: the thermal equator of each month's zonal-mean T̄.
        for (m, it) in itcz.iter_mut().enumerate() {
            let rt: Vec<f64> = (0..h).map(|j| row_t[j][m] / w as f64).collect();
            *it = thermal_equator(g, &rt, p.itcz_max_deg);
        }
        out.itcz = itcz;
        let annual: Vec<f64> = (0..n)
            .map(|c| {
                (0..12)
                    .map(|m| f[c] * out.t_land[c][m] + (1.0 - f[c]) * out.t_ocean[c][m])
                    .sum::<f64>()
                    / 12.0
            })
            .collect();
        let ice = state.ice_j_m2.clone();
        if let Some((pa, pi)) = &prev
            && pa.iter().zip(&annual).all(|(a, b)| (a - b).abs() < 0.005)
            && pi
                .iter()
                .zip(&ice)
                .all(|(a, b)| (a - b).abs() < 0.02 * ICE_J_M3)
        {
            break;
        }
        prev = Some((annual, ice));
    }
    out
}

fn northward(w: usize, k: &[f64], t: &[f64], scale: f64, acc: &mut [f64]) {
    for (e, a) in acc.iter_mut().enumerate() {
        let s: f64 = (0..w)
            .map(|i| {
                let c = e * w + i;
                k[c] * (t[c + w] - t[c])
            })
            .sum();
        *a += s * scale;
    }
}

fn diffuse_x(w: usize, h: usize, v: &[f64], k: &[f64], cap: &[f64], dt: f64) -> Vec<f64> {
    let rows: Vec<Vec<f64>> = (0..h)
        .into_par_iter()
        .map(|j| {
            let r0 = j * w;
            let mut lo = vec![0.0; w];
            let mut di = vec![0.0; w];
            let mut up = vec![0.0; w];
            for i in 0..w {
                let c = r0 + i;
                let ke = k[c] * dt / cap[c];
                let kw = k[r0 + (i + w - 1) % w] * dt / cap[c];
                lo[i] = -kw;
                up[i] = -ke;
                di[i] = 1.0 + ke + kw;
            }
            if lo.iter().all(|&x| x == 0.0) {
                return v[r0..r0 + w].to_vec();
            }
            cyclic_thomas(&lo, &di, &up, &v[r0..r0 + w])
        })
        .collect();
    rows.concat()
}

fn diffuse_y(w: usize, h: usize, v: &[f64], k: &[f64], cap: &[f64], dt: f64) -> Vec<f64> {
    let cols: Vec<Vec<f64>> = (0..w)
        .into_par_iter()
        .map(|i| {
            let mut lo = vec![0.0; h];
            let mut di = vec![0.0; h];
            let mut up = vec![0.0; h];
            let mut r = vec![0.0; h];
            for j in 0..h {
                let c = j * w + i;
                let ks = k[c] * dt / cap[c];
                let kn = if j > 0 { k[c - w] * dt / cap[c] } else { 0.0 };
                lo[j] = -kn;
                up[j] = -ks;
                di[j] = 1.0 + ks + kn;
                r[j] = v[c];
            }
            crate::climate::ebm::thomas(&lo, &di, &up, &r)
        })
        .collect();
    let mut out = vec![0.0; w * h];
    for (i, col) in cols.into_iter().enumerate() {
        for (j, x) in col.into_iter().enumerate() {
            out[j * w + i] = x;
        }
    }
    out
}
