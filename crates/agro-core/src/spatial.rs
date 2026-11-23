/// Inverse Distance Weighting (IDW) point interpolation.
pub fn idw_point(pts_x: &[f64], pts_y: &[f64], pts_val: &[f64], power: f64, tx: f64, ty: f64) -> f64 {
    let n = pts_x.len().min(pts_y.len()).min(pts_val.len());
    if n == 0 {
        return 0.0;
    }
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        let dx = tx - pts_x[i];
        let dy = ty - pts_y[i];
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < 1e-18 {
            return pts_val[i];
        }
        let dist = dist_sq.sqrt();
        let w = 1.0 / dist.powf(power);
        num += w * pts_val[i];
        den += w;
    }
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

/// Simple OLS regression y = slope * x + intercept.
/// Returns (slope, intercept, r2, sample_guard_triggered) where guard triggers when n < 30.
pub fn ols_simple(x: &[f64], y: &[f64]) -> (f64, f64, f64, bool) {
    let n = x.len().min(y.len());
    let guard = n < 30;
    if n == 0 {
        return (0.0, 0.0, 0.0, true);
    }
    let nf = n as f64;
    let mean_x = x[..n].iter().sum::<f64>() / nf;
    let mean_y = y[..n].iter().sum::<f64>() / nf;
    if n < 2 {
        return (0.0, mean_y, 0.0, true);
    }

    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let mut syy = 0.0;
    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }

    if sxx < 1e-12 {
        return (0.0, mean_y, 0.0, guard);
    }

    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    let r2 = if syy < 1e-12 {
        1.0
    } else {
        ((sxy * sxy) / (sxx * syy)).clamp(0.0, 1.0)
    };
    (slope, intercept, r2, guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idw_exact_match_and_average() {
        let px = [0.0, 10.0];
        let py = [0.0, 0.0];
        let pv = [4.0, 8.0];
        assert!((idw_point(&px, &py, &pv, 2.0, 0.0, 0.0) - 4.0).abs() < 1e-9);
        assert!((idw_point(&px, &py, &pv, 2.0, 5.0, 0.0) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_ols_guard_triggers_under_30_samples() {
        let x: Vec<f64> = (0..12).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| 7.0 - 0.1 * v).collect();
        let (slope, intercept, r2, guard) = ols_simple(&x, &y);
        assert!(guard, "Guard must trigger when n=12 < 30");
        assert!((slope - (-0.1)).abs() < 1e-9);
        assert!((intercept - 7.0).abs() < 1e-9);
        assert!((r2 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_ols_guard_clears_at_36_samples() {
        let x: Vec<f64> = (0..36).map(|i| 20.0 + i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| 8.5 - 0.08 * v).collect();
        let (slope, _, r2, guard) = ols_simple(&x, &y);
        assert!(!guard, "Guard must not trigger when n=36 >= 30");
        assert!((slope - (-0.08)).abs() < 1e-9);
        assert!((r2 - 1.0).abs() < 1e-9);
    }
}
