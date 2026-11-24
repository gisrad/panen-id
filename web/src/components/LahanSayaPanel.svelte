<script lang="ts">
  import {
    computeFao56Series,
    computeGddSeries,
    computeIdwGrid,
    computeOlsSummary,
    computePhenologySeries,
    CROP_PROFILES,
    type CropKey,
    type OlsSummary,
  } from '../lib/agroWasm';

  interface ByodRow {
    plot_id: string;
    lon: number;
    lat: number;
    clay_pct: number;
    yield_t_ha: number;
    ndvi: number;
  }

  let rows: ByodRow[] = [];
  let sourceLabel = 'Belum ada berkas lokal dimuat';
  let isSimulatedPreset = false;
  let ols: OlsSummary | null = null;
  let idwGridMinMax: { min: number; max: number; mean: number } | null = null;
  let busy = false;
  let errorMsg = '';

  const HUB_PRESETS: { label: string; lat: number; lon: number }[] = [
    { label: 'DAS Bompon', lat: -7.55, lon: 110.07 },
    { label: 'Karawang', lat: -6.31, lon: 107.3 },
    { label: 'Deli Serdang', lat: 3.55, lon: 98.85 },
    { label: 'Sidrap', lat: -3.93, lon: 119.79 },
    { label: 'Merauke', lat: -8.49, lon: 140.4 },
  ];
  let weatherLat = -7.55;
  let weatherLon = 110.07;
  let weatherStart = '2025-05-01';
  let weatherEnd = '2025-08-28';
  let weatherCrop: CropKey = 'padi';
  let weatherBusy = false;
  let weatherErr = '';
  let weatherSummary: {
    days: number;
    rainMm: number;
    meanTmax: number;
    wasmGdd: number;
    wasmEt0Mm: number;
    waterBalanceMm: number;
    finalStage: string;
    cached: boolean;
    source: string;
  } | null = null;

  function buildSampleRows(count: number): ByodRow[] {
    const out: ByodRow[] = [];
    for (let i = 0; i < count; i++) {
      const col = i % 6;
      const row = Math.floor(i / 6);
      const clay = 28 + ((i * 7) % 36);
      // Higher claypan percentage suppresses yield cleanly with modest noise
      const noise = ((i * 13) % 7 - 3) * 0.08;
      const yld = Math.max(2.6, Number((8.1 - 0.068 * clay + noise).toFixed(2)));
      const ndvi = Number((0.46 + (yld - 2.6) * 0.065).toFixed(3));
      out.push({
        plot_id: `LOK-${String(i + 1).padStart(2, '0')}`,
        lon: Number((109.96 + col * 0.0025).toFixed(5)),
        lat: Number((-7.552 + row * 0.0022).toFixed(5)),
        clay_pct: clay,
        yield_t_ha: yld,
        ndvi,
      });
    }
    return out;
  }

  async function runWasmDiagnostics(data: ByodRow[]) {
    busy = true;
    errorMsg = '';
    try {
      const clayArr = data.map((r) => r.clay_pct);
      const yieldArr = data.map((r) => r.yield_t_ha);
      ols = await computeOlsSummary(clayArr, yieldArr);

      const lonArr = data.map((r) => r.lon);
      const latArr = data.map((r) => r.lat);
      const minX = Math.min(...lonArr);
      const maxX = Math.max(...lonArr);
      const minY = Math.min(...latArr);
      const maxY = Math.max(...latArr);
      const gx: number[] = [];
      const gy: number[] = [];
      for (let ix = 0; ix < 4; ix++) {
        for (let iy = 0; iy < 4; iy++) {
          gx.push(minX + ((maxX - minX) || 0.01) * (ix / 3));
          gy.push(minY + ((maxY - minY) || 0.01) * (iy / 3));
        }
      }
      const gridVals = await computeIdwGrid(lonArr, latArr, yieldArr, gx, gy, 2.0);
      if (gridVals.length > 0) {
        const min = Math.min(...gridVals);
        const max = Math.max(...gridVals);
        const mean = gridVals.reduce((a, b) => a + b, 0) / gridVals.length;
        idwGridMinMax = { min, max, mean };
      }
      if (lonArr.length > 0 && latArr.length > 0) {
        const cLon = lonArr.reduce((a, b) => a + b, 0) / lonArr.length;
        const cLat = latArr.reduce((a, b) => a + b, 0) / latArr.length;
        if (cLat >= -11.5 && cLat <= 6.5 && cLon >= 94.5 && cLon <= 141.5) {
          weatherLat = Number(cLat.toFixed(4));
          weatherLon = Number(cLon.toFixed(4));
        }
      }
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : 'Gagal menjalankan kernel agro_core.wasm';
    } finally {
      busy = false;
    }
  }

  function dayOfYear(isoDate: string): number {
    const dt = new Date(`${isoDate}T00:00:00Z`);
    if (Number.isNaN(dt.getTime())) return 150;
    const start = Date.UTC(dt.getUTCFullYear(), 0, 0);
    return Math.max(1, Math.min(366, Math.round((dt.getTime() - start) / 86400000)));
  }

  async function fetchEra5AndComputeWasm() {
    weatherBusy = true;
    weatherErr = '';
    try {
      const qs = new URLSearchParams({
        lat: String(weatherLat),
        lon: String(weatherLon),
        start: weatherStart,
        end: weatherEnd,
      });
      const res = await fetch(`/api/v1/weather?${qs.toString()}`);
      if (res.status === 401) {
        weatherErr = 'Sesi pratinjau 5 menit habis. Masuk dengan Google untuk menarik cuaca ERA5 langsung.';
        return;
      }
      const data = await res.json();
      if (!res.ok) {
        weatherErr = data?.error === 'invalid_indonesia_bbox'
          ? 'Koordinat berada di luar batas Indonesia (94.5°–141.5° BT, 11.5° LS–6.5° LU).'
          : `Gagal menarik data cuaca (${data?.error || res.status}).`;
        return;
      }
      const profile = CROP_PROFILES[weatherCrop];
      const tmax: number[] = data.tmax_c || [];
      const tmin: number[] = data.tmin_c || [];
      const rain: number[] = data.rain_mm || [];
      const rh: number[] = data.rh_pct || [];
      const wind: number[] = data.wind_ms || [];
      const rs: number[] = data.rs_mj || [];
      const dates: string[] = data.dates || [];
      const doy = dates.map(dayOfYear);

      const gddArr = await computeGddSeries(tmax, tmin, profile.tbase, profile.tupper);
      const stageArr = await computePhenologySeries(gddArr, profile.code);
      const et0Arr = await computeFao56Series(tmax, tmin, rh, wind, rs, doy, weatherLat, 120);

      const totalRain = rain.reduce((a, b) => a + b, 0);
      const meanTmax = tmax.length ? tmax.reduce((a, b) => a + b, 0) / tmax.length : 0;
      const finalGdd = gddArr.length ? gddArr[gddArr.length - 1] : 0;
      const totalEt0 = et0Arr.reduce((a, b) => a + b, 0);
      const lastStageIdx = stageArr.length ? Math.round(stageArr[stageArr.length - 1]) : 0;

      weatherSummary = {
        days: dates.length,
        rainMm: Number(totalRain.toFixed(1)),
        meanTmax: Number(meanTmax.toFixed(1)),
        wasmGdd: Number(finalGdd.toFixed(0)),
        wasmEt0Mm: Number(totalEt0.toFixed(1)),
        waterBalanceMm: Number((totalRain - totalEt0).toFixed(1)),
        finalStage: profile.stages[Math.min(5, Math.max(0, lastStageIdx))],
        cached: Boolean(data.cached),
        source: String(data.source || 'Open-Meteo ERA5 Archive'),
      };
    } catch (err) {
      weatherErr = err instanceof Error ? err.message : 'Gagal menghitung GDD & ET0 di WASM.';
    } finally {
      weatherBusy = false;
    }
  }

  async function loadPreset(count: 12 | 36) {
    isSimulatedPreset = true;
    sourceLabel = `SIMULASI PRESET BYOD (${count} petak)`;
    rows = buildSampleRows(count);
    await runWasmDiagnostics(rows);
  }

  async function handleFile(ev: Event) {
    const input = ev.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    errorMsg = '';
    isSimulatedPreset = false;
    sourceLabel = `Berkas lokal: ${file.name} (tidak diunggah ke server)`;
    try {
      const text = await file.text();
      if (file.name.toLowerCase().endsWith('.json') || file.name.toLowerCase().endsWith('.geojson')) {
        const gj = JSON.parse(text);
        const features = Array.isArray(gj.features) ? gj.features : [];
        rows = features
          .map((f: any, idx: number): ByodRow | null => {
            const p = f.properties || {};
            const coords = f.geometry?.type === 'Point' ? f.geometry.coordinates : f.geometry?.coordinates?.[0]?.[0];
            const lon = Number(p.lon ?? coords?.[0] ?? 109.96 + idx * 0.001);
            const lat = Number(p.lat ?? coords?.[1] ?? -7.55 + idx * 0.001);
            const clay = Number(p.clay_pct ?? p.clay ?? 40);
            const yld = Number(p.yield_t_ha ?? p.yield ?? NaN);
            const ndvi = Number(p.ndvi ?? p.ndvi_peak ?? 0.68);
            if (!Number.isFinite(yld)) return null;
            return {
              plot_id: String(p.plot_id ?? `F-${idx + 1}`),
              lon,
              lat,
              clay_pct: clay,
              yield_t_ha: yld,
              ndvi,
            };
          })
          .filter((r: ByodRow | null): r is ByodRow => r !== null);
      } else {
        const lines = text
          .split(/\r?\n/)
          .map((l) => l.trim())
          .filter(Boolean);
        if (lines.length < 2) throw new Error('CSV kosong atau tanpa baris data.');
        const header = lines[0].split(',').map((h) => h.trim().toLowerCase());
        const iPlot = header.findIndex((h) => h.includes('plot') || h === 'id');
        const iLon = header.findIndex((h) => h.includes('lon') || h === 'x');
        const iLat = header.findIndex((h) => h.includes('lat') || h === 'y');
        const iClay = header.findIndex((h) => h.includes('clay') || h.includes('liat'));
        const iYield = header.findIndex((h) => h.includes('yield') || h.includes('hasil') || h.includes('ton'));
        const iNdvi = header.findIndex((h) => h.includes('ndvi'));
        if (iYield < 0) throw new Error('Kolom yield/hasil wajib ada di CSV.');
        rows = lines
          .slice(1)
          .map((line, idx): ByodRow | null => {
            const cols = line.split(',').map((c) => c.trim());
            const yld = Number(cols[iYield]);
            if (!Number.isFinite(yld)) return null;
            return {
              plot_id: iPlot >= 0 && cols[iPlot] ? cols[iPlot] : `CSV-${idx + 1}`,
              lon: iLon >= 0 && Number.isFinite(Number(cols[iLon])) ? Number(cols[iLon]) : 109.96 + idx * 0.001,
              lat: iLat >= 0 && Number.isFinite(Number(cols[iLat])) ? Number(cols[iLat]) : -7.55 + idx * 0.001,
              clay_pct: iClay >= 0 && Number.isFinite(Number(cols[iClay])) ? Number(cols[iClay]) : 42,
              yield_t_ha: yld,
              ndvi: iNdvi >= 0 && Number.isFinite(Number(cols[iNdvi])) ? Number(cols[iNdvi]) : 0.66,
            };
          })
          .filter((r: ByodRow | null): r is ByodRow => r !== null);
      }

      if (rows.length === 0) {
        throw new Error('Tidak ada baris numerik valid yang dapat diparsing.');
      }
      await runWasmDiagnostics(rows);
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : 'Gagal membaca berkas lokal.';
    }
  }

  function exportCsv() {
    if (rows.length === 0) return;
    const meta = [
      `# source: ${sourceLabel}`,
      `# attribution: Agri Transbulent (agri.transbulent.id) — analisis lokal Rust WASM agro-core`,
      `# simulated: ${isSimulatedPreset ? 'SIMULASI' : 'DATA_PENGGUNA_LOKAL'}`,
      `# n_samples: ${rows.length} | sample_guard_n_lt_30: ${ols?.sampleGuardTriggered ? 'TRUE' : 'FALSE'}`,
    ];
    const header = 'plot_id,lon,lat,clay_pct,yield_t_ha,ndvi';
    const body = rows.map((r) => `${r.plot_id},${r.lon},${r.lat},${r.clay_pct},${r.yield_t_ha},${r.ndvi}`);
    const blob = new Blob([[...meta, header, ...body].join('\n')], { type: 'text/csv;charset=utf-8' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'lahan-saya-analisis-wasm.csv';
    a.click();
    URL.revokeObjectURL(a.href);
  }

  function exportGeoJson() {
    if (rows.length === 0) return;
    const fc = {
      type: 'FeatureCollection',
      metadata: {
        source: sourceLabel,
        attribution: 'Agri Transbulent (agri.transbulent.id) — analisis lokal Rust WASM agro-core',
        simulated: isSimulatedPreset,
        sample_guard_n_lt_30: Boolean(ols?.sampleGuardTriggered),
      },
      features: rows.map((r) => ({
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [r.lon, r.lat] },
        properties: {
          plot_id: r.plot_id,
          clay_pct: r.clay_pct,
          yield_t_ha: r.yield_t_ha,
          ndvi: r.ndvi,
          is_simulated: isSimulatedPreset,
        },
      })),
    };
    const blob = new Blob([JSON.stringify(fc, null, 2)], { type: 'application/geo+json;charset=utf-8' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'lahan-saya-analisis-wasm.geojson';
    a.click();
    URL.revokeObjectURL(a.href);
  }
</script>

<div style="display: grid; gap: 12px;">
  <div style="border: 1px solid var(--border); padding: 14px; background: var(--surface); display: grid; gap: 10px;">
    <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px;">
      <span class="pill-badge">LOCAL-FIRST · RUST WASM AGRO-CORE</span>
      {#if isSimulatedPreset}
        <span class="pill-badge pill-warn">SIMULASI</span>
      {/if}
    </div>
    <p style="margin: 0; font-size: 0.88rem; color: var(--ink-secondary);">
      Unggah CSV atau GeoJSON petak panen Anda. Seluruh perhitungan OLS &amp; IDW berjalan di memori peramban melalui <code>agro_core.wasm</code> tanpa mengirim data mentah ke server. Mendukung koordinat seluruh wilayah Indonesia (95&deg;&ndash;141&deg; BT, 6&deg; LU&ndash;11&deg; LS).
    </p>

    <div style="display: flex; flex-wrap: wrap; gap: 8px; align-items: center;">
      <label class="btn-primary" style="cursor: pointer; font-size: 12px;">
        Pilih CSV / GeoJSON
        <input type="file" accept=".csv,.geojson,.json" on:change={handleFile} style="display: none;" />
      </label>
      <button type="button" class="btn-subtle" on:click={() => loadPreset(12)}>
        Contoh 12 Petak (Uji Guard)
      </button>
      <button type="button" class="btn-subtle" on:click={() => loadPreset(36)}>
        Contoh 36 Petak (Valid)
      </button>
    </div>

    <div class="num" style="font-size: 11px; color: var(--ink-muted);">
      Sumber aktif: {sourceLabel}
    </div>
  </div>

  {#if errorMsg}
    <div style="border: 1px solid #B83B14; padding: 12px; color: #B83B14; font-size: 0.88rem;">
      {errorMsg}
    </div>
  {/if}

  {#if busy}
    <div class="num" style="font-size: 12px; color: var(--ink-muted);">Menghitung di WebAssembly...</div>
  {/if}

  {#if ols}
    {#if ols.sampleGuardTriggered}
      <div style="border: 1.5px solid #B83B14; padding: 12px; background: var(--surface); display: grid; gap: 4px;">
        <div class="num" style="font-size: 11px; font-weight: 600; color: #B83B14;">
          PERINGATAN STATISTIK (n = {ols.n} &lt; 30 SAMPEL)
        </div>
        <p style="margin: 0; font-size: 0.86rem;">
          Jumlah petak kurang dari ambang minimum inferensi 30 sampel. Gunakan statistik deskriptif (rata-rata &amp; rentang IDW) saja sebelum menarik kesimpulan kausal.
        </p>
      </div>
    {:else}
      <div style="border: 1.5px solid #1E6B47; padding: 12px; background: var(--surface); display: grid; gap: 4px;">
        <div class="num" style="font-size: 11px; font-weight: 600; color: #1E6B47;">
          INFERENSI OLS VALID (n = {ols.n} &ge; 30 SAMPEL)
        </div>
        <p style="margin: 0; font-size: 0.86rem;">
          Hubungan Liat (%) vs Hasil Panen (t/ha): kemiringan <strong>{ols.slope.toFixed(4)}</strong> t/ha per % liat (R&sup2; = <strong>{ols.r2.toFixed(3)}</strong>).
        </p>
      </div>
    {/if}

    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;">
      <div style="border: 1px solid var(--border); padding: 10px;">
        <div class="num" style="font-size: 10px; color: var(--ink-muted);">SAMPEL (N)</div>
        <div class="num" style="font-size: 1.2rem; font-weight: 600;">{ols.n}</div>
      </div>
      <div style="border: 1px solid var(--border); padding: 10px;">
        <div class="num" style="font-size: 10px; color: var(--ink-muted);">OLS R&sup2;</div>
        <div class="num" style="font-size: 1.2rem; font-weight: 600;">{ols.r2.toFixed(3)}</div>
      </div>
      <div style="border: 1px solid var(--border); padding: 10px;">
        <div class="num" style="font-size: 10px; color: var(--ink-muted);">IDW RATA-RATA</div>
        <div class="num" style="font-size: 1.2rem; font-weight: 600;">
          {idwGridMinMax ? idwGridMinMax.mean.toFixed(2) : '-'} t/ha
        </div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; gap: 6px; flex-wrap: wrap;">
      <span class="num" style="font-size: 11px; color: var(--ink-muted);">Pratinjau 6 baris pertama</span>
      <div style="display: flex; gap: 6px;">
        <button type="button" class="btn-subtle" on:click={exportCsv}>Unduh CSV + Atribusi</button>
        <button type="button" class="btn-subtle" on:click={exportGeoJson}>Unduh GeoJSON (QGIS)</button>
      </div>
    </div>

    <div style="border: 1px solid var(--border); overflow-x: auto;">
      <table style="width: 100%; border-collapse: collapse; font-size: 11.5px;">
        <thead>
          <tr style="border-bottom: 1px solid var(--border); text-align: left;">
            <th style="padding: 6px 8px;">Petak</th>
            <th style="padding: 6px 8px;">Liat %</th>
            <th style="padding: 6px 8px;">Hasil t/ha</th>
            <th style="padding: 6px 8px;">NDVI</th>
          </tr>
        </thead>
        <tbody>
          {#each rows.slice(0, 6) as row}
            <tr style="border-bottom: 1px solid var(--border);">
              <td class="num" style="padding: 5px 8px;">{row.plot_id}</td>
              <td class="num" style="padding: 5px 8px;">{row.clay_pct}</td>
              <td class="num" style="padding: 5px 8px;">{row.yield_t_ha}</td>
              <td class="num" style="padding: 5px 8px;">{row.ndvi}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <div style="border: 1px solid var(--border-ink); padding: 14px; background: var(--surface); display: grid; gap: 10px;">
    <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; flex-wrap: wrap;">
      <span class="num" style="font-size: 10.5px; font-weight: 600;">
        TARIK CUACA ERA5 NUSANTARA &amp; HITUNG GDD/ET0 (WASM)
      </span>
      <span class="pill-badge">INDONESIA · /api/v1/weather</span>
    </div>

    <div style="display: flex; flex-wrap: wrap; gap: 5px;">
      {#each HUB_PRESETS as p}
        <button
          type="button"
          class={Math.abs(weatherLat - p.lat) < 0.05 && Math.abs(weatherLon - p.lon) < 0.05 ? 'btn-primary' : 'btn-subtle'}
          style="padding: 4px 7px; font-size: 10px;"
          on:click={() => {
            weatherLat = p.lat;
            weatherLon = p.lon;
          }}
        >
          {p.label}
        </button>
      {/each}
    </div>

    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(115px, 1fr)); gap: 8px;">
      <label style="display: grid; gap: 3px; font-size: 10.5px;">
        <span class="num" style="color: var(--ink-muted);">LAT (°)</span>
        <input type="number" step="0.01" min="-11.5" max="6.5" bind:value={weatherLat} style="padding: 5px 6px; border: 1px solid var(--border-ink); background: var(--bg); color: var(--ink); font-family: inherit;" />
      </label>
      <label style="display: grid; gap: 3px; font-size: 10.5px;">
        <span class="num" style="color: var(--ink-muted);">LON (°)</span>
        <input type="number" step="0.01" min="94.5" max="141.5" bind:value={weatherLon} style="padding: 5px 6px; border: 1px solid var(--border-ink); background: var(--bg); color: var(--ink); font-family: inherit;" />
      </label>
      <label style="display: grid; gap: 3px; font-size: 10.5px;">
        <span class="num" style="color: var(--ink-muted);">MULAI</span>
        <input type="date" bind:value={weatherStart} style="padding: 5px 6px; border: 1px solid var(--border-ink); background: var(--bg); color: var(--ink); font-family: inherit;" />
      </label>
      <label style="display: grid; gap: 3px; font-size: 10.5px;">
        <span class="num" style="color: var(--ink-muted);">SELESAI</span>
        <input type="date" bind:value={weatherEnd} style="padding: 5px 6px; border: 1px solid var(--border-ink); background: var(--bg); color: var(--ink); font-family: inherit;" />
      </label>
      <label style="display: grid; gap: 3px; font-size: 10.5px;">
        <span class="num" style="color: var(--ink-muted);">KOMODITAS</span>
        <select bind:value={weatherCrop} style="padding: 5px 6px; border: 1px solid var(--border-ink); background: var(--bg); color: var(--ink); font-family: inherit;">
          <option value="padi">Padi Sawah</option>
          <option value="jagung">Jagung Hibrida</option>
          <option value="kedelai">Kedelai</option>
          <option value="bawang">Bawang Merah</option>
          <option value="tebu">Tebu</option>
        </select>
      </label>
    </div>

    <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
      <button type="button" class="btn-primary" style="font-size: 11.5px;" disabled={weatherBusy} on:click={fetchEra5AndComputeWasm}>
        {weatherBusy ? 'Menarik ERA5 & Menghitung WASM...' : 'Tarik ERA5 & Hitung WASM GDD/ET0'}
      </button>
      {#if weatherSummary}
        <span class="num" style="font-size: 10.5px; color: var(--ink-muted);">
          {weatherSummary.days} hari · {weatherSummary.source} ({weatherSummary.cached ? 'KV Cache' : 'Live'})
        </span>
      {/if}
    </div>

    {#if weatherErr}
      <div style="border: 1px solid #B83B14; padding: 8px 10px; color: #B83B14; font-size: 0.84rem;">
        {weatherErr}
      </div>
    {/if}

    {#if weatherSummary}
      <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 1px; background: var(--border); border: 1px solid var(--border);">
        <div style="background: var(--bg); padding: 8px 10px;">
          <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">GDD KUMULATIF · FASE AKHIR</div>
          <div class="num" style="font-size: 1.08rem; font-weight: 600;">
            {weatherSummary.wasmGdd} · {weatherSummary.finalStage}
          </div>
        </div>
        <div style="background: var(--bg); padding: 8px 10px;">
          <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">FAO-56 ET0 WASM</div>
          <div class="num" style="font-size: 1.08rem; font-weight: 600;">{weatherSummary.wasmEt0Mm} mm</div>
        </div>
        <div style="background: var(--bg); padding: 8px 10px;">
          <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">TOTAL HUJAN ERA5 · TMAX RATA2</div>
          <div class="num" style="font-size: 1.08rem; font-weight: 600;">
            {weatherSummary.rainMm} mm · {weatherSummary.meanTmax}°C
          </div>
        </div>
        <div style="background: var(--bg); padding: 8px 10px;">
          <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">NERACA AIR (HUJAN − ET0)</div>
          <div class="num" style="font-size: 1.08rem; font-weight: 600; color: {weatherSummary.waterBalanceMm >= 0 ? '#1E6B47' : '#B83B14'};">
            {weatherSummary.waterBalanceMm >= 0 ? '+' : ''}{weatherSummary.waterBalanceMm} mm
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
