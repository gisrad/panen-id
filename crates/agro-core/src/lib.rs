pub mod fao56;
pub mod gdd;
pub mod ml;
pub mod spatial;

use std::slice;

/// Allocate a buffer of `len` f64 values in WASM linear memory for host-side writes.
#[no_mangle]
pub extern "C" fn alloc_f64(len: usize) -> *mut f64 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    let mut buf = vec![0.0_f64; len];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free a previously allocated `alloc_f64` buffer.
///
/// # Safety
/// `ptr` must have been returned by `alloc_f64(len)` and not yet freed.
#[no_mangle]
pub unsafe extern "C" fn dealloc_f64(ptr: *mut f64, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

/// Compute cumulative GDD into `out_ptr[0..len]`.
///
/// # Safety
/// All three pointers must reference valid arrays of length `len`.
#[no_mangle]
pub unsafe extern "C" fn wasm_gdd_series(
    tmax_ptr: *const f64,
    tmin_ptr: *const f64,
    len: usize,
    tbase: f64,
    tupper: f64,
    out_ptr: *mut f64,
) {
    if tmax_ptr.is_null() || tmin_ptr.is_null() || out_ptr.is_null() || len == 0 {
        return;
    }
    let tmax = slice::from_raw_parts(tmax_ptr, len);
    let tmin = slice::from_raw_parts(tmin_ptr, len);
    let out = slice::from_raw_parts_mut(out_ptr, len);
    gdd::cumulative_gdd(tmax, tmin, tbase, tupper, out);
}

/// Compute FAO-56 Penman-Monteith daily ET0 (mm/day) into `out_ptr[0..len]`.
///
/// # Safety
/// All array pointers must reference valid arrays of length `len`.
#[no_mangle]
pub unsafe extern "C" fn wasm_fao56_et0_series(
    tmax_ptr: *const f64,
    tmin_ptr: *const f64,
    rh_ptr: *const f64,
    u2_ptr: *const f64,
    rs_ptr: *const f64,
    doy_ptr: *const f64,
    len: usize,
    lat_deg: f64,
    elev_m: f64,
    out_ptr: *mut f64,
) {
    if tmax_ptr.is_null()
        || tmin_ptr.is_null()
        || rh_ptr.is_null()
        || u2_ptr.is_null()
        || rs_ptr.is_null()
        || doy_ptr.is_null()
        || out_ptr.is_null()
        || len == 0
    {
        return;
    }
    let tmax = slice::from_raw_parts(tmax_ptr, len);
    let tmin = slice::from_raw_parts(tmin_ptr, len);
    let rh = slice::from_raw_parts(rh_ptr, len);
    let u2 = slice::from_raw_parts(u2_ptr, len);
    let rs = slice::from_raw_parts(rs_ptr, len);
    let doy = slice::from_raw_parts(doy_ptr, len);
    let out = slice::from_raw_parts_mut(out_ptr, len);

    for i in 0..len {
        out[i] = fao56::fao56_et0_day(tmax[i], tmin[i], rh[i], u2[i], rs[i], doy[i], lat_deg, elev_m);
    }
}

/// Compute simple OLS regression y = slope*x + intercept.
/// Writes `[slope, intercept, r2, guard_flag]` (4 f64 values) into `out_ptr`.
///
/// # Safety
/// `x_ptr` and `y_ptr` must hold `len` elements; `out_ptr` must hold at least 4 elements.
#[no_mangle]
pub unsafe extern "C" fn wasm_ols_summary(
    x_ptr: *const f64,
    y_ptr: *const f64,
    len: usize,
    out_ptr: *mut f64,
) {
    if x_ptr.is_null() || y_ptr.is_null() || out_ptr.is_null() {
        return;
    }
    let x = slice::from_raw_parts(x_ptr, len);
    let y = slice::from_raw_parts(y_ptr, len);
    let out = slice::from_raw_parts_mut(out_ptr, 4);
    let (slope, intercept, r2, guard) = spatial::ols_simple(x, y);
    out[0] = slope;
    out[1] = intercept;
    out[2] = r2;
    out[3] = if guard { 1.0 } else { 0.0 };
}

/// Compute IDW interpolation across `n_grid` target coordinates into `out_ptr[0..n_grid]`.
///
/// # Safety
/// Input sample pointers must have `n_pts` elements; grid/out pointers must have `n_grid` elements.
#[no_mangle]
pub unsafe extern "C" fn wasm_idw_grid(
    pts_x: *const f64,
    pts_y: *const f64,
    pts_val: *const f64,
    n_pts: usize,
    power: f64,
    grid_x: *const f64,
    grid_y: *const f64,
    n_grid: usize,
    out_ptr: *mut f64,
) {
    if pts_x.is_null()
        || pts_y.is_null()
        || pts_val.is_null()
        || grid_x.is_null()
        || grid_y.is_null()
        || out_ptr.is_null()
        || n_pts == 0
        || n_grid == 0
    {
        return;
    }
    let px = slice::from_raw_parts(pts_x, n_pts);
    let py = slice::from_raw_parts(pts_y, n_pts);
    let pv = slice::from_raw_parts(pts_val, n_pts);
    let gx = slice::from_raw_parts(grid_x, n_grid);
    let gy = slice::from_raw_parts(grid_y, n_grid);
    let out = slice::from_raw_parts_mut(out_ptr, n_grid);

    for i in 0..n_grid {
        out[i] = spatial::idw_point(px, py, pv, power, gx[i], gy[i]);
    }
}

/// Compute phenology stage series (`0.0..=5.0`) from cumulative GDD values.
///
/// # Safety
/// `gdd_ptr` and `out_ptr` must reference valid arrays of length `len`.
#[no_mangle]
pub unsafe extern "C" fn wasm_phenology_series(
    gdd_ptr: *const f64,
    len: usize,
    crop_code: i32,
    out_ptr: *mut f64,
) {
    if gdd_ptr.is_null() || out_ptr.is_null() || len == 0 {
        return;
    }
    let gdd = slice::from_raw_parts(gdd_ptr, len);
    let out = slice::from_raw_parts_mut(out_ptr, len);
    gdd::phenology_series(gdd, crop_code, out);
}

/// Partition fields into 3 standardized K-Means management zones (0=low, 1=mid, 2=high).
/// Returns mean Silhouette separation score.
///
/// # Safety
/// Input pointers and `out_labels_ptr` must reference arrays of length `len`.
#[no_mangle]
pub unsafe extern "C" fn wasm_kmeans_zones(
    yields_ptr: *const f64,
    clays_ptr: *const f64,
    ndvis_ptr: *const f64,
    len: usize,
    out_labels_ptr: *mut f64,
) -> f64 {
    if yields_ptr.is_null() || clays_ptr.is_null() || ndvis_ptr.is_null() || out_labels_ptr.is_null() || len == 0 {
        return 0.0;
    }
    let yields = slice::from_raw_parts(yields_ptr, len);
    let clays = slice::from_raw_parts(clays_ptr, len);
    let ndvis = slice::from_raw_parts(ndvis_ptr, len);
    let out = slice::from_raw_parts_mut(out_labels_ptr, len);
    ml::kmeans_3_zones(yields, clays, ndvis, out)
}

/// Multivariate standardized Ridge regression `y = b0 + b1*z_clay + b2*z_ndvi + b3*z_perm`.
/// Writes `[b1, b2, b3, imp_clay_pct, imp_ndvi_pct, imp_perm_pct, r2, rmse, guard, mean_y]` into `out_10_ptr`.
///
/// # Safety
/// Input pointers must have length `len` and `out_10_ptr` must hold at least 10 `f64` values.
#[no_mangle]
pub unsafe extern "C" fn wasm_explain_yield(
    clays_ptr: *const f64,
    ndvis_ptr: *const f64,
    perms_ptr: *const f64,
    yields_ptr: *const f64,
    len: usize,
    out_10_ptr: *mut f64,
) {
    if clays_ptr.is_null() || ndvis_ptr.is_null() || perms_ptr.is_null() || yields_ptr.is_null() || out_10_ptr.is_null() {
        return;
    }
    let clays = slice::from_raw_parts(clays_ptr, len);
    let ndvis = slice::from_raw_parts(ndvis_ptr, len);
    let perms = slice::from_raw_parts(perms_ptr, len);
    let yields = slice::from_raw_parts(yields_ptr, len);
    let out = slice::from_raw_parts_mut(out_10_ptr, 10);
    out.copy_from_slice(&ml::explain_yield_ridge(clays, ndvis, perms, yields));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_abi_alloc_and_gdd() {
        unsafe {
            let n = 4;
            let tmax = alloc_f64(n);
            let tmin = alloc_f64(n);
            let out = alloc_f64(n);
            std::slice::from_raw_parts_mut(tmax, n).copy_from_slice(&[30.0, 31.0, 32.0, 29.0]);
            std::slice::from_raw_parts_mut(tmin, n).copy_from_slice(&[22.0, 23.0, 22.0, 21.0]);
            wasm_gdd_series(tmax, tmin, n, 10.0, 35.0, out);
            let res = std::slice::from_raw_parts(out, n);
            assert!((res[0] - 16.0).abs() < 1e-9);
            assert!((res[3] - 65.0).abs() < 1e-9);
            dealloc_f64(tmax, n);
            dealloc_f64(tmin, n);
            dealloc_f64(out, n);
        }
    }

    #[test]
    fn test_c_abi_idw_and_ols() {
        unsafe {
            let px = [0.0, 10.0];
            let py = [0.0, 0.0];
            let pv = [5.0, 9.0];
            let gx = [5.0];
            let gy = [0.0];
            let mut out = [0.0];
            wasm_idw_grid(px.as_ptr(), py.as_ptr(), pv.as_ptr(), 2, 2.0, gx.as_ptr(), gy.as_ptr(), 1, out.as_mut_ptr());
            assert!((out[0] - 7.0).abs() < 1e-9);

            let mut ols_out = [0.0; 4];
            wasm_ols_summary(px.as_ptr(), pv.as_ptr(), 2, ols_out.as_mut_ptr());
            assert!((ols_out[0] - 0.4).abs() < 1e-9);
            assert_eq!(ols_out[3], 1.0);
        }
    }

    #[test]
    fn test_c_abi_phenology_series() {
        unsafe {
            let gdd = [100.0, 450.0, 920.0, 1600.0];
            let mut out = [0.0; 4];
            wasm_phenology_series(gdd.as_ptr(), 4, 0, out.as_mut_ptr());
            assert_eq!(out, [0.0, 1.0, 3.0, 5.0]);
            wasm_phenology_series(gdd.as_ptr(), 4, 2, out.as_mut_ptr());
            assert_eq!(out, [1.0, 2.0, 4.0, 5.0]);
            wasm_phenology_series(gdd.as_ptr(), 4, 3, out.as_mut_ptr());
            assert_eq!(out, [0.0, 2.0, 4.0, 5.0]);
            wasm_phenology_series(gdd.as_ptr(), 4, 4, out.as_mut_ptr());
            assert_eq!(out, [0.0, 1.0, 2.0, 3.0]);
        }
    }

    #[test]
    fn test_c_abi_ml_kmeans_and_explain() {
        unsafe {
            let yields = [3.0, 3.2, 5.1, 5.3, 7.2, 7.4];
            let clays = [62.0, 60.0, 44.0, 42.0, 28.0, 26.0];
            let ndvis = [0.50, 0.52, 0.66, 0.68, 0.80, 0.82];
            let perms = [4.0, 4.5, 9.0, 9.5, 16.0, 17.0];
            let mut labels = [0.0; 6];
            let sil = wasm_kmeans_zones(yields.as_ptr(), clays.as_ptr(), ndvis.as_ptr(), 6, labels.as_mut_ptr());
            assert!(sil > 0.3);
            assert_eq!(labels[0], 0.0);
            assert_eq!(labels[5], 2.0);

            let mut exp = [0.0; 10];
            wasm_explain_yield(clays.as_ptr(), ndvis.as_ptr(), perms.as_ptr(), yields.as_ptr(), 6, exp.as_mut_ptr());
            assert!(exp[6] > 0.95);
            assert!((exp[3] + exp[4] + exp[5] - 100.0).abs() < 1e-6);
        }
    }
}
