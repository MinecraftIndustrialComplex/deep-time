// SPDX-License-Identifier: LGPL-3.0-or-later
//! Permissive tracts (plan §8.6 step 2; Singer 1993's first step): connected regions of L1 cells
//! whose favourability is above the type's threshold, eight-connected, wrapping east–west.

use deeptime_format::GridDesc;

use super::fields::cell_area_km2;

/// One tract.
#[derive(Clone, Debug, PartialEq)]
pub struct Tract {
    /// L1 cells, ascending.
    pub cells: Vec<u32>,
    /// True area (km²).
    pub area_km2: f64,
    /// Area-weighted mean favourability.
    pub mean_f: f64,
}

/// Connected components of cells with `f[cell] > threshold`.
pub fn find_tracts(grid: &GridDesc, f: &[f64], threshold: f64, m_per_block_h: f64) -> Vec<Tract> {
    let (w, h) = (grid.w as usize, grid.h as usize);
    let mut seen = vec![false; f.len()];
    let area_row: Vec<f64> = (0..grid.h)
        .map(|j| cell_area_km2(grid, j, m_per_block_h))
        .collect();
    let mut out = Vec::new();
    let mut stack = Vec::new();
    for start in 0..f.len() {
        if seen[start] || f[start] <= threshold {
            continue;
        }
        seen[start] = true;
        stack.push(start);
        let mut cells = Vec::new();
        while let Some(c) = stack.pop() {
            cells.push(c as u32);
            let (i, j) = (c % w, c / w);
            for dj in -1i64..=1 {
                let jj = j as i64 + dj;
                if jj < 0 || jj >= h as i64 {
                    continue;
                }
                for di in -1i64..=1 {
                    if di == 0 && dj == 0 {
                        continue;
                    }
                    let ii = (i as i64 + di).rem_euclid(w as i64) as usize;
                    let n = jj as usize * w + ii;
                    if !seen[n] && f[n] > threshold {
                        seen[n] = true;
                        stack.push(n);
                    }
                }
            }
        }
        cells.sort_unstable();
        let mut area = 0.0;
        let mut weighted = 0.0;
        for &c in &cells {
            let a = area_row[c as usize / w];
            area += a;
            weighted += a * f[c as usize];
        }
        let mean_f = if area > 0.0 { weighted / area } else { 0.0 };
        out.push(Tract {
            cells,
            area_km2: area,
            mean_f,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_wrap_the_seam_and_respect_the_threshold() {
        let grid = GridDesc::for_planet("L1", 256, 16).unwrap(); // 16 × 8 cells
        let mut f = vec![0.0; grid.cells()];
        // A band crossing the seam in row 3, and a separate blob.
        for i in [14usize, 15, 0, 1] {
            f[3 * 16 + i] = 0.8;
        }
        f[6 * 16 + 7] = 0.5;
        f[6 * 16 + 8] = 0.1; // below the threshold
        let t = find_tracts(&grid, &f, 0.2, 300.0);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].cells, vec![48, 49, 62, 63]);
        assert!((t[0].mean_f - 0.8).abs() < 1e-12);
        assert_eq!(t[1].cells, vec![6 * 16 + 7]);
        assert!(t[0].area_km2 > 0.0);
    }
}
