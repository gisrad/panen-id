/// Daily modified Growing Degree Days (GDD) using temperature clamping.
pub fn daily_gdd(tmax: f64, tmin: f64, tbase: f64, tupper: f64) -> f64 {
    let tmax_c = tmax.max(tbase).min(tupper);
    let tmin_c = tmin.max(tbase).min(tupper);
    let avg = 0.5 * (tmax_c + tmin_c);
    (avg - tbase).max(0.0)
}

/// Fills `out` with cumulative GDD from daily `tmax` and `tmin` arrays.
pub fn cumulative_gdd(tmax: &[f64], tmin: &[f64], tbase: f64, tupper: f64, out: &mut [f64]) {
    let n = tmax.len().min(tmin.len()).min(out.len());
    let mut acc = 0.0;
    for i in 0..n {
        acc += daily_gdd(tmax[i], tmin[i], tbase, tupper);
        out[i] = acc;
    }
}

/// Returns integer phenology stage index for rice (crop_code=0) or maize (crop_code=1).
pub fn phenology_stage(gdd_cum: f64, crop_code: i32) -> i32 {
    if crop_code == 1 {
        // Maize (Jagung)
        if gdd_cum < 80.0 {
            0
        } else if gdd_cum < 350.0 {
            1
        } else if gdd_cum < 650.0 {
            2
        } else if gdd_cum < 820.0 {
            3
        } else if gdd_cum < 1250.0 {
            4
        } else {
            5
        }
    } else {
        // Rice (Padi) default
        if gdd_cum < 120.0 {
            0
        } else if gdd_cum < 500.0 {
            1
        } else if gdd_cum < 900.0 {
            2
        } else if gdd_cum < 1200.0 {
            3
        } else if gdd_cum < 1550.0 {
            4
        } else {
            5
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_gdd_clamping() {
        // Rice base 10 C, upper 35 C
        assert!((daily_gdd(32.0, 22.0, 10.0, 35.0) - 17.0).abs() < 1e-9);
        // Below base should yield 0
        assert!((daily_gdd(9.0, 5.0, 10.0, 35.0) - 0.0).abs() < 1e-9);
        // Heat capping at 35 C
        assert!((daily_gdd(42.0, 20.0, 10.0, 35.0) - 17.5).abs() < 1e-9);
    }

    #[test]
    fn test_cumulative_and_phenology() {
        let tmax = [30.0, 32.0, 31.0];
        let tmin = [22.0, 24.0, 21.0];
        let mut out = [0.0; 3];
        cumulative_gdd(&tmax, &tmin, 10.0, 35.0, &mut out);
        assert!((out[0] - 16.0).abs() < 1e-9);
        assert!((out[1] - 34.0).abs() < 1e-9);
        assert!((out[2] - 50.0).abs() < 1e-9);
        assert_eq!(phenology_stage(50.0, 0), 0);
        assert_eq!(phenology_stage(950.0, 0), 3);
        assert_eq!(phenology_stage(700.0, 1), 3);
    }
}
