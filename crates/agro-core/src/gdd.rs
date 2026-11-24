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

/// Returns integer phenology stage index (0..=5) for Indonesian crops:
/// `0` = Padi (Rice), `1` = Jagung (Maize), `2` = Kedelai (Soybean),
/// `3` = Bawang Merah / Hortikultura (Shallot), `4` = Tebu (Sugarcane).
pub fn phenology_stage(gdd_cum: f64, crop_code: i32) -> i32 {
    let cuts = match crop_code {
        1 => [80.0, 350.0, 650.0, 820.0, 1250.0],
        2 => [90.0, 340.0, 620.0, 860.0, 1120.0],
        3 => [110.0, 300.0, 540.0, 760.0, 960.0],
        4 => [220.0, 780.0, 1450.0, 2100.0, 2750.0],
        _ => [120.0, 500.0, 900.0, 1200.0, 1550.0],
    };
    for (idx, threshold) in cuts.iter().enumerate() {
        if gdd_cum < *threshold {
            return idx as i32;
        }
    }
    5
}

/// Fills `out` with phenology stage index (`0.0..=5.0`) for each cumulative GDD value.
pub fn phenology_series(gdd_cum: &[f64], crop_code: i32, out: &mut [f64]) {
    let n = gdd_cum.len().min(out.len());
    for i in 0..n {
        out[i] = phenology_stage(gdd_cum[i], crop_code) as f64;
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

    #[test]
    fn test_multi_commodity_phenology_stages() {
        // Kedelai (code 2)
        assert_eq!(phenology_stage(60.0, 2), 0);
        assert_eq!(phenology_stage(400.0, 2), 2);
        assert_eq!(phenology_stage(1200.0, 2), 5);
        // Bawang Merah (code 3)
        assert_eq!(phenology_stage(100.0, 3), 0);
        assert_eq!(phenology_stage(600.0, 3), 3);
        assert_eq!(phenology_stage(1000.0, 3), 5);
        // Tebu (code 4)
        assert_eq!(phenology_stage(200.0, 4), 0);
        assert_eq!(phenology_stage(1600.0, 4), 3);
        assert_eq!(phenology_stage(2800.0, 4), 5);

        let gdds = [50.0, 350.0, 980.0];
        let mut stages = [0.0; 3];
        phenology_series(&gdds, 3, &mut stages);
        assert_eq!(stages, [0.0, 2.0, 5.0]);
    }
}
