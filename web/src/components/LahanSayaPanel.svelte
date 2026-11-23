<script lang="ts">
  import { computeIdwGrid, computeOlsSummary, type OlsSummary } from '../lib/agroWasm';

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
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : 'Gagal menjalankan kernel agro_core.wasm';
    } finally {
      busy = false;
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
</div>
