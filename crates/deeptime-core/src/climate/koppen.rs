// SPDX-License-Identifier: LGPL-3.0-or-later
//! Köppen–Geiger classification from 12 monthly temperatures and precipitations (plan §8.3
//! item 5), with the thresholds of Peel, Finlayson & McMahon 2007 (Table 1; the rules the research
//! note cites) and the common extension that splits As from Aw by the season of the driest month.
//!
//! - E if the warmest month is below 10 °C (ET above 0 °C, EF otherwise) — checked first, so cold
//!   deserts are polar, not arid.
//! - B if MAP < 10·P_th, with P_th = 2·MAT, + 28 when ≥ 70 % of the precipitation falls in the
//!   summer half, + 14 otherwise (+ 0 when ≥ 70 % falls in winter); BW below 5·P_th; h if
//!   MAT ≥ 18 °C, else k.
//! - A if the coldest month is ≥ 18 °C: f if the driest month has ≥ 60 mm, m if ≥ 100 − MAP/25,
//!   else w (s when the driest month is in summer).
//! - C if the coldest month is in (0, 18) °C, D if ≤ 0 °C: s if the driest summer month has
//!   < 40 mm and < a third of the wettest winter month, w if the driest winter month has less than
//!   a tenth of the wettest summer month, else f; a if the warmest month ≥ 22 °C, b if at least four
//!   months ≥ 10 °C, d (D only) if the coldest month < −38 °C, else c.
//!
//! The summer half is whichever of April–September and October–March is warmer (so it follows
//! the hemisphere and any planet's tilt).

/// Class codes, by index (the value of the `climate/koppen` layer; 31 = sea).
pub const CODES: [&str; 32] = [
    "Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc",
    "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb",
    "Dfc", "Dfd", "ET", "EF", "sea",
];

/// The layer value of the sea.
pub const SEA: u8 = 31;

/// The index of a code.
pub fn index(code: &str) -> Option<u8> {
    CODES.iter().position(|&c| c == code).map(|i| i as u8)
}

/// The main group letter of a class index (`'A'`…`'E'`, `'-'` for the sea).
pub fn group(k: u8) -> char {
    match k {
        0..=3 => 'A',
        4..=7 => 'B',
        8..=16 => 'C',
        17..=28 => 'D',
        29 | 30 => 'E',
        _ => '-',
    }
}

/// Classifies monthly mean temperatures (°C) and monthly precipitation totals (mm).
#[allow(clippy::needless_range_loop)]
pub fn classify(t: &[f64; 12], p: &[f64; 12]) -> u8 {
    let mat = t.iter().sum::<f64>() / 12.0;
    let map: f64 = p.iter().sum();
    let thot = t.iter().cloned().fold(f64::MIN, f64::max);
    let tcold = t.iter().cloned().fold(f64::MAX, f64::min);
    // Summer half: Apr–Sep (months 3..9) or Oct–Mar, whichever is warmer.
    let apr_sep: f64 = t[3..9].iter().sum();
    let summer_is_apr_sep = apr_sep >= t.iter().sum::<f64>() - apr_sep;
    let in_summer = |m: usize| (3..9).contains(&m) == summer_is_apr_sep;
    let (mut ps, mut pw) = (0.0, 0.0);
    let (mut psdry, mut pswet, mut pwdry, mut pwwet) = (f64::MAX, 0.0f64, f64::MAX, 0.0f64);
    for m in 0..12 {
        if in_summer(m) {
            ps += p[m];
            psdry = psdry.min(p[m]);
            pswet = pswet.max(p[m]);
        } else {
            pw += p[m];
            pwdry = pwdry.min(p[m]);
            pwwet = pwwet.max(p[m]);
        }
    }
    let code = if thot < 10.0 {
        if thot > 0.0 { "ET" } else { "EF" }
    } else {
        let pth = if map > 0.0 && pw >= 0.7 * map {
            2.0 * mat
        } else if map > 0.0 && ps >= 0.7 * map {
            2.0 * mat + 28.0
        } else {
            2.0 * mat + 14.0
        };
        if map < 10.0 * pth {
            match (map < 5.0 * pth, mat >= 18.0) {
                (true, true) => "BWh",
                (true, false) => "BWk",
                (false, true) => "BSh",
                (false, false) => "BSk",
            }
        } else if tcold >= 18.0 {
            let pdry = p.iter().cloned().fold(f64::MAX, f64::min);
            if pdry >= 60.0 {
                "Af"
            } else if pdry >= 100.0 - map / 25.0 {
                "Am"
            } else {
                let driest = (0..12)
                    .min_by(|&a, &b| p[a].total_cmp(&p[b]).then(a.cmp(&b)))
                    .unwrap_or(0);
                if in_summer(driest) { "As" } else { "Aw" }
            }
        } else {
            let second = if psdry < 40.0 && psdry < pwwet / 3.0 {
                's'
            } else if pwdry < pswet / 10.0 {
                'w'
            } else {
                'f'
            };
            let warm_months = t.iter().filter(|&&x| x >= 10.0).count();
            let third = if thot >= 22.0 {
                'a'
            } else if warm_months >= 4 {
                'b'
            } else if tcold > 0.0 || tcold >= -38.0 {
                'c'
            } else {
                'd'
            };
            let first = if tcold > 0.0 { 'C' } else { 'D' };
            let third = if first == 'C' && third == 'd' {
                'c'
            } else {
                third
            };
            let s = format!("{first}{second}{third}");
            return index(&s).expect("a valid code");
        }
    };
    index(code).expect("a valid code")
}

/// Legend colours, by index (the palette of Beck et al. 2018's Köppen–Geiger maps; the sea dark
/// blue).
pub const COLOURS: [&str; 32] = [
    "#0000ff", "#0078ff", "#46aafa", "#7fbfff", "#ff0000", "#ff9696", "#f5a500", "#ffdc64",
    "#ffff00", "#c8c800", "#969600", "#96ff96", "#64c864", "#329632", "#c8ff50", "#64ff50",
    "#32c800", "#ff00ff", "#c800c8", "#963296", "#966496", "#aaafff", "#5a78dc", "#4b50b4",
    "#320087", "#00ffff", "#37c8ff", "#007d7d", "#00465f", "#b2b2b2", "#666666", "#0b1d3a",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn code(t: [f64; 12], p: [f64; 12]) -> &'static str {
        CODES[classify(&t, &p) as usize]
    }

    #[test]
    fn hand_built_climates_classify() {
        // Af — Singapore-like: hot, wet every month.
        let af = code(
            [
                26.5, 27.0, 27.5, 28.0, 28.2, 28.0, 27.7, 27.6, 27.4, 27.3, 26.9, 26.4,
            ],
            [
                240.0, 160.0, 185.0, 180.0, 170.0, 130.0, 155.0, 150.0, 165.0, 155.0, 255.0, 290.0,
            ],
        );
        assert_eq!(af, "Af");
        // BWh — Riyadh-like: hot and nearly rainless.
        let bwh = code(
            [
                14.0, 17.0, 21.0, 26.0, 32.0, 35.0, 36.0, 36.0, 33.0, 27.0, 20.0, 15.0,
            ],
            [
                15.0, 8.0, 22.0, 25.0, 7.0, 0.0, 0.0, 0.0, 0.0, 1.0, 6.0, 13.0,
            ],
        );
        assert_eq!(bwh, "BWh");
        // Cfb — London-like.
        let cfb = code(
            [
                5.0, 5.3, 7.6, 9.9, 13.3, 16.5, 18.7, 18.5, 15.7, 12.0, 8.0, 5.5,
            ],
            [
                55.0, 41.0, 42.0, 44.0, 49.0, 45.0, 45.0, 50.0, 49.0, 69.0, 59.0, 55.0,
            ],
        );
        assert_eq!(cfb, "Cfb");
        // Dfc — Arkhangelsk-like: long cold winters, three months ≥ 10 °C.
        let dfc = code(
            [
                -12.0, -11.0, -5.0, 1.0, 7.0, 13.0, 16.5, 14.0, 8.0, 2.0, -4.0, -9.0,
            ],
            [
                35.0, 28.0, 27.0, 28.0, 40.0, 60.0, 70.0, 70.0, 60.0, 55.0, 45.0, 40.0,
            ],
        );
        assert_eq!(dfc, "Dfc");
        // ET — Svalbard-like: warmest month above 0 °C but below 10 °C.
        let et = code(
            [
                -14.0, -15.0, -14.5, -11.0, -4.0, 2.0, 6.0, 5.0, 1.0, -5.0, -9.0, -12.0,
            ],
            [
                15.0, 13.0, 16.0, 11.0, 10.0, 9.0, 18.0, 22.0, 20.0, 17.0, 14.0, 17.0,
            ],
        );
        assert_eq!(et, "ET");
        // The southern hemisphere: a Perth-like Csa has its dry summer in December–February.
        let csa = code(
            [
                24.5, 24.8, 23.0, 19.8, 16.5, 14.2, 13.3, 13.8, 15.2, 17.5, 20.5, 22.8,
            ],
            [
                15.0, 10.0, 20.0, 40.0, 110.0, 170.0, 150.0, 120.0, 80.0, 50.0, 25.0, 12.0,
            ],
        );
        assert_eq!(csa, "Csa");
        // EF, Aw, Dwd.
        assert_eq!(code([-30.0; 12], [5.0; 12]), "EF");
        let aw = code(
            [
                25.0, 26.0, 28.0, 30.0, 29.0, 27.0, 26.0, 26.0, 27.0, 27.0, 26.0, 25.0,
            ],
            [
                5.0, 5.0, 10.0, 40.0, 150.0, 250.0, 300.0, 300.0, 200.0, 80.0, 10.0, 5.0,
            ],
        );
        assert_eq!(aw, "Aw");
        let dwd = code(
            [
                -46.0, -42.0, -30.0, -12.0, 3.0, 13.0, 16.0, 12.0, 3.0, -14.0, -34.0, -44.0,
            ],
            [
                5.0, 4.0, 3.0, 6.0, 12.0, 35.0, 45.0, 40.0, 20.0, 10.0, 8.0, 5.0,
            ],
        );
        assert_eq!(dwd, "Dwd");
        assert_eq!(group(index("Cfb").unwrap()), 'C');
        assert_eq!(CODES.len(), COLOURS.len());
    }
}
