use std::f64::consts::PI;

/// FAO-56 Penman-Monteith daily reference evapotranspiration ET0 (mm/day).
/// Implements Allen et al. (1998) Eq. 6 for a 0.12 m clipped grass reference surface.
pub fn fao56_et0_day(
    tmax: f64,
    tmin: f64,
    rh_mean: f64,
    u2: f64,
    rs_mj: f64,
    doy: f64,
    lat_deg: f64,
    elev_m: f64,
) -> f64 {
    let t_mean = 0.5 * (tmax + tmin);
    let p = 101.3 * ((293.0 - 0.0065 * elev_m) / 293.0).powf(5.26);
    let gamma = 0.000665 * p;

    let es_tmax = 0.6108 * ((17.27 * tmax) / (tmax + 237.3)).exp();
    let es_tmin = 0.6108 * ((17.27 * tmin) / (tmin + 237.3)).exp();
    let es = 0.5 * (es_tmax + es_tmin);
    let ea = (es * (rh_mean.clamp(5.0, 100.0) / 100.0)).min(es);

    let delta = 4098.0 * (0.6108 * ((17.27 * t_mean) / (t_mean + 237.3)).exp())
        / (t_mean + 237.3).powi(2);

    // Extraterrestrial radiation Ra (MJ m-2 day-1)
    let lat_rad = lat_deg * PI / 180.0;
    let dr = 1.0 + 0.033 * ((2.0 * PI / 365.0) * doy).cos();
    let decl = 0.409 * ((2.0 * PI / 365.0) * doy - 1.39).sin();
    let ws = (-lat_rad.tan() * decl.tan()).clamp(-1.0, 1.0).acos();
    let ra = (24.0 * 60.0 / PI)
        * 0.0820
        * dr
        * (ws * lat_rad.sin() * decl.sin() + lat_rad.cos() * decl.cos() * ws.sin());

    let rso = (0.75 + 2e-5 * elev_m) * ra.max(1e-6);
    let rs_rso = (rs_mj / rso).clamp(0.3, 1.0);
    let rns = 0.77 * rs_mj;

    let tmax_k4 = (tmax + 273.16).powi(4);
    let tmin_k4 = (tmin + 273.16).powi(4);
    let sigma = 4.903e-9;
    let rnl = sigma * 0.5 * (tmax_k4 + tmin_k4) * (0.34 - 0.14 * ea.sqrt()) * (1.35 * rs_rso - 0.35);
    let rn = rns - rnl;
    let g = 0.0; // Daily soil heat flux ~ 0

    let u2_safe = u2.max(0.2);
    let num = 0.408 * delta * (rn - g) + gamma * (900.0 / (t_mean + 273.0)) * u2_safe * (es - ea);
    let den = delta + gamma * (1.0 + 0.34 * u2_safe);
    (num / den).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fao56_tropical_bompon() {
        // Typical sunny tropical Central Java day: Tmax 31, Tmin 23, RH 75%, u2 1.8 m/s, Rs 20 MJ/m2/d
        let et0 = fao56_et0_day(31.0, 23.0, 75.0, 1.8, 20.0, 180.0, -7.5, 180.0);
        assert!(et0 > 3.5 && et0 < 6.5, "Unexpected tropical ET0: {et0}");
    }

    #[test]
    fn test_fao56_increases_with_wind_and_radiation() {
        let calm_cloudy = fao56_et0_day(28.0, 22.0, 82.0, 0.8, 12.0, 45.0, -7.6, 150.0);
        let windy_sunny = fao56_et0_day(33.0, 22.0, 60.0, 3.0, 24.0, 240.0, -7.6, 150.0);
        assert!(windy_sunny > calm_cloudy + 1.5);
    }
}
