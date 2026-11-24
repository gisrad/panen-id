/// Feature-standardized 3-zone K-Means clustering on `(yield, clay, ndvi)`.
/// Remaps cluster IDs by ascending mean yield:
/// `0` = low yield (`Kritis / Restorasi`), `1` = mid (`Transisi`), `2` = high (`Potensi Tinggi`).
/// Returns mean Silhouette separation score `[-1..1]`.
pub fn kmeans_3_zones(yields: &[f64], clays: &[f64], ndvis: &[f64], out_labels: &mut [f64]) -> f64 {
    let n = yields.len().min(clays.len()).min(ndvis.len()).min(out_labels.len());
    if n == 0 {
        return 0.0;
    }
    if n < 3 {
        for v in out_labels[..n].iter_mut() {
            *v = 1.0;
        }
        return 0.0;
    }

    let z_y = standardize(&yields[..n]);
    let z_c = standardize(&clays[..n]);
    let z_n = standardize(&ndvis[..n]);

    // Deterministic quantile-seeded centroids based on sorted yields (25th, 50th, 75th percentiles)
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| yields[a].total_cmp(&yields[b]));
    let seeds = [order[n / 6], order[n / 2], order[(n * 5) / 6]];
    let mut cx = [[0.0_f64; 3]; 3];
    for (k, &idx) in seeds.iter().enumerate() {
        cx[k] = [z_y[idx], z_c[idx], z_n[idx]];
    }

    let mut assign = vec![0usize; n];
    for _iter in 0..30 {
        let mut changed = false;
        for i in 0..n {
            let mut best_k = 0usize;
            let mut best_d = f64::INFINITY;
            for (k, c) in cx.iter().enumerate() {
                let d = sq_dist3([z_y[i], z_c[i], z_n[i]], *c);
                if d < best_d {
                    best_d = d;
                    best_k = k;
                }
            }
            if assign[i] != best_k {
                assign[i] = best_k;
                changed = true;
            }
        }

        let mut sums = [[0.0_f64; 3]; 3];
        let mut counts = [0usize; 3];
        for i in 0..n {
            let k = assign[i];
            sums[k][0] += z_y[i];
            sums[k][1] += z_c[i];
            sums[k][2] += z_n[i];
            counts[k] += 1;
        }
        for k in 0..3 {
            if counts[k] > 0 {
                let cf = counts[k] as f64;
                cx[k] = [sums[k][0] / cf, sums[k][1] / cf, sums[k][2] / cf];
            }
        }
        if !changed {
            break;
        }
    }

    // Order clusters by ascending mean yield so 0=worst, 1=mid, 2=best
    let mut yield_sum = [0.0_f64; 3];
    let mut count = [0usize; 3];
    for i in 0..n {
        let k = assign[i];
        yield_sum[k] += yields[i];
        count[k] += 1;
    }
    let mut rank_pairs: Vec<(usize, f64)> = (0..3)
        .map(|k| (k, if count[k] > 0 { yield_sum[k] / count[k] as f64 } else { f64::INFINITY }))
        .collect();
    rank_pairs.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut remap = [0usize; 3];
    for (new_id, &(old_id, _)) in rank_pairs.iter().enumerate() {
        remap[old_id] = new_id;
    }

    for i in 0..n {
        assign[i] = remap[assign[i]];
        out_labels[i] = assign[i] as f64;
    }

    // Silhouette-like separation score [-1..1]
    let mut sil_sum = 0.0;
    for i in 0..n {
        let own = assign[i];
        let mut own_d = 0.0;
        let mut own_cnt = 0usize;
        let mut other_sums = [0.0_f64; 3];
        let mut other_cnts = [0usize; 3];
        for j in 0..n {
            if i == j {
                continue;
            }
            let d = sq_dist3([z_y[i], z_c[i], z_n[i]], [z_y[j], z_c[j], z_n[j]]).sqrt();
            if assign[j] == own {
                own_d += d;
                own_cnt += 1;
            } else {
                other_sums[assign[j]] += d;
                other_cnts[assign[j]] += 1;
            }
        }
        let a = if own_cnt > 0 { own_d / own_cnt as f64 } else { 0.0 };
        let b = (0..3)
            .filter(|&k| k != own && other_cnts[k] > 0)
            .map(|k| other_sums[k] / other_cnts[k] as f64)
            .fold(f64::INFINITY, f64::min);
        let s = if b.is_finite() && (a > 0.0 || b > 0.0) {
            (b - a) / a.max(b)
        } else {
            0.0
        };
        sil_sum += s;
    }
    (sil_sum / n as f64).clamp(-1.0, 1.0)
}

/// Multivariate standardized Ridge regression: `y = b0 + b1*z_clay + b2*z_ndvi + b3*z_perm`.
/// Solves 4x4 normal equations via Gaussian elimination with partial pivoting (`lambda = 1e-4` on non-intercept terms).
/// Returns `[b1, b2, b3, imp_clay_pct, imp_ndvi_pct, imp_perm_pct, r2, rmse, sample_guard, mean_y]`.
pub fn explain_yield_ridge(clays: &[f64], ndvis: &[f64], perms: &[f64], yields: &[f64]) -> [f64; 10] {
    let n = clays.len().min(ndvis.len()).min(perms.len()).min(yields.len());
    let guard = if n < 30 { 1.0 } else { 0.0 };
    if n == 0 {
        return [0.0, 0.0, 0.0, 33.4, 33.3, 33.3, 0.0, 0.0, 1.0, 0.0];
    }
    let nf = n as f64;
    let mean_y = yields[..n].iter().sum::<f64>() / nf;
    if n < 4 {
        return [0.0, 0.0, 0.0, 33.4, 33.3, 33.3, 0.0, 0.0, 1.0, mean_y];
    }

    let z_c = standardize(&clays[..n]);
    let z_n = standardize(&ndvis[..n]);
    let z_p = standardize(&perms[..n]);

    // Build normal equation matrix A (4x4) and RHS vector b (4)
    let mut a = [[0.0_f64; 4]; 4];
    let mut b = [0.0_f64; 4];
    for i in 0..n {
        let row = [1.0, z_c[i], z_n[i], z_p[i]];
        let yi = yields[i];
        for r in 0..4 {
            b[r] += row[r] * yi;
            for c in 0..4 {
                a[r][c] += row[r] * row[c];
            }
        }
    }
    let lambda = 1e-4;
    for k in 1..4 {
        a[k][k] += lambda;
    }

    let beta = solve_4x4(a, b);
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for i in 0..n {
        let y_hat = beta[0] + beta[1] * z_c[i] + beta[2] * z_n[i] + beta[3] * z_p[i];
        let err = yields[i] - y_hat;
        let dev = yields[i] - mean_y;
        ss_res += err * err;
        ss_tot += dev * dev;
    }
    let r2 = if ss_tot < 1e-12 {
        1.0
    } else {
        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    };
    let rmse = (ss_res / nf).sqrt();

    let abs_sum = beta[1].abs() + beta[2].abs() + beta[3].abs();
    let (ic, i_n, ip) = if abs_sum > 1e-12 {
        (
            (beta[1].abs() / abs_sum) * 100.0,
            (beta[2].abs() / abs_sum) * 100.0,
            (beta[3].abs() / abs_sum) * 100.0,
        )
    } else {
        (33.4, 33.3, 33.3)
    };

    [beta[1], beta[2], beta[3], ic, i_n, ip, r2, rmse, guard, mean_y]
}

fn standardize(xs: &[f64]) -> Vec<f64> {
    let nf = xs.len() as f64;
    let mu = xs.iter().sum::<f64>() / nf;
    let var = xs.iter().map(|v| (v - mu) * (v - mu)).sum::<f64>() / nf;
    let sd = var.sqrt().max(1e-6);
    xs.iter().map(|v| (v - mu) / sd).collect()
}

fn sq_dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d0 = a[0] - b[0];
    let d1 = a[1] - b[1];
    let d2 = a[2] - b[2];
    d0 * d0 + d1 * d1 + d2 * d2
}

fn solve_4x4(mut a: [[f64; 4]; 4], mut b: [f64; 4]) -> [f64; 4] {
    for col in 0..4 {
        let mut piv = col;
        for row in (col + 1)..4 {
            if a[row][col].abs() > a[piv][col].abs() {
                piv = row;
            }
        }
        a.swap(col, piv);
        b.swap(col, piv);

        let diag = a[col][col];
        if diag.abs() < 1e-12 {
            continue;
        }
        for row in (col + 1)..4 {
            let factor = a[row][col] / diag;
            b[row] -= factor * b[col];
            for k in col..4 {
                a[row][k] -= factor * a[col][k];
            }
        }
    }
    let mut x = [0.0_f64; 4];
    for row in (0..4).rev() {
        let mut s = b[row];
        for col in (row + 1)..4 {
            s -= a[row][col] * x[col];
        }
        x[row] = if a[row][row].abs() >= 1e-12 { s / a[row][row] } else { 0.0 };
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans_3_zones_separates_yield_tiers() {
        let yields = [3.1, 3.3, 3.4, 5.2, 5.4, 5.5, 7.2, 7.5, 7.6];
        let clays = [62.0, 60.0, 58.0, 44.0, 42.0, 40.0, 28.0, 26.0, 25.0];
        let ndvis = [0.51, 0.53, 0.52, 0.67, 0.69, 0.68, 0.82, 0.84, 0.83];
        let mut labels = [0.0; 9];
        let sil = kmeans_3_zones(&yields, &clays, &ndvis, &mut labels);
        assert!(sil > 0.4, "Expected clear cluster separation, got {sil}");
        assert_eq!(labels[0], 0.0);
        assert_eq!(labels[4], 1.0);
        assert_eq!(labels[8], 2.0);
    }

    #[test]
    fn test_explain_yield_ridge_recovers_dominant_driver() {
        let n = 36;
        let clays: Vec<f64> = (0..n).map(|i| 25.0 + (i as f64) * 1.1).collect();
        let ndvis: Vec<f64> = (0..n).map(|i| 0.72 + ((i % 3) as f64) * 0.01).collect();
        let perms: Vec<f64> = (0..n).map(|i| 12.0 + ((i % 4) as f64) * 0.2).collect();
        let yields: Vec<f64> = clays.iter().map(|c| 8.8 - 0.075 * c).collect();
        let out = explain_yield_ridge(&clays, &ndvis, &perms, &yields);
        assert!(out[0] < -0.5, "Negative beta for clay expected, got {}", out[0]);
        assert!(out[3] > 60.0, "Clay importance should dominate, got {}", out[3]);
        assert!(out[6] > 0.9, "High R2 expected, got {}", out[6]);
        assert_eq!(out[8], 0.0, "Sample guard should clear at n=36");
    }
}
