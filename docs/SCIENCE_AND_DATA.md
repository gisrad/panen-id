# Metodologi Ilmiah & Kontrak Data — Panen-ID

## 1. Masukan Hasil Panen (Konteks Indonesia)
1. **Jalur Utama**: hasil per petak (poligon), titik ubinan (metode BPS), catatan panen petani, dan proksi satelit (integral NDVI/EVI musim) terkalibrasi terhadap titik ubinan.
2. **Jalur Sekunder**: unggahan *combine harvester* (`CSV`, `Shapefile .zip`, `ISOXML`).
3. Data sintetis/contoh wajib menandai `is_simulated: true` dan menampilkan badge **`SIMULASI`** di seluruh antarmuka serta ekspor.

## 2. Rumus Agronomi & Iklim Mikro (`crates/agro-core`)
- **Thermal Time (GDD Harian)**:
  $$\text{GDD}_d = \max\!\left(0,\; \frac{\min(T_{\max}, T_{\text{cap}}) + \max(T_{\min}, T_{\text{base}})}{2} - T_{\text{base}}\right)$$
  - Padi: $T_{\text{base}} = 10\,^\circ\text{C}$, $T_{\text{cap}} = 35\,^\circ\text{C}$ (dapat dikonfigurasi). Fase: *Bibit → Anakan → Bunting → Heading → Pengisian → Masak*.
  - Jagung: $T_{\text{base}} = 10\,^\circ\text{C}$, $T_{\text{cap}} = 30\,^\circ\text{C}$ (metode 86/50 °F). Fase: *VE → Vn → VT → R1 → R6*.
- **Neraca Air FAO-56 (Model Bucket)**:
  - $\text{ET}_c = K_c(\text{fase}) \times \text{ET}_0$ (`et0_fao_evapotranspiration` dari Open-Meteo).
  - $\text{TAW}$ diturunkan dari kapasitas lapang & titik layu melalui fungsi pedotransfer SoilGrids (liat, pasir, bahan organik). Deplesi $> \text{RAW} = p \times \text{TAW}$ menandakan stres kekeringan.
- **Jendela Stres Berbasis Fase**:
  - **Stres Panas**: $T_{\max} \ge 35\,^\circ\text{C}$ dalam rentang $\pm 7$ hari dari *heading* (padi) atau VT–R1 (jagung).
  - **Kemarau Kering**: $\ge 10$ hari berturut-turut dengan hujan harian $< 1\,\text{mm}$.

## 3. Penginderaan Jauh (Optik + SAR)
- **Sentinel-2 L2A NDVI**: $\text{NDVI} = (\text{B08} - \text{B04}) / (\text{B08} + \text{B04})$, masking kelas `SCL`: `0, 1, 3, 8, 9, 10`.
- > [!CAUTION]
  > **Pitfall Processing Baseline 04.00**: Mulai 25 Januari 2022, produk Sentinel-2 L2A menyertakan **`BOA_ADD_OFFSET = -1000`**. Pipeline wajib mengoreksi $\text{reflectance} = (\text{DN} + \text{BOA\_ADD\_OFFSET}) / 10000$ sebelum menghitung NDVI/EVI agar deret waktu lintas tahun konsisten.
- **Gabungan Multi-Sensor**: gabungkan Sentinel-2 L2A + **NASA HLS 30 m**, haluskan dengan *Whittaker / Savitzky–Golay*, ekstrak SOS / puncak / EOS. Gunakan **Sentinel-1 VH RTC** untuk mendeteksi penggenangan awal tanam padi di bawah tutupan awan.

## 4. Pemodelan "Lensa Dalam" & Guard Statistik
- **Lensa Dangkal** (`hasil ~ varietas`) vs **Lensa Dalam** (OLS dengan interaksi `varietas × zona` + *feature importance* Random Forest).
- **Stabilitas**: koefisien variasi (CV) hasil ternormalisasi lintas musim; **Zona Manajemen**: *k-means* pada vektor hasil + tanah terstandar.
- **Guard Sampel Kecil**: jika $n < 30$, nonaktifkan regresi inferensial dan tampilkan statistik deskriptif saja beserta peringatan.
- **Insight Deterministik**: dibangun dari template terukur (bukan LLM bebas), misalnya: *"Varietas X di zona liat berat turun 18 % (±6 %) saat defisit air 21 hari sebelum berbunga"*.

## 5. Kontrak R2 (`/v1/season/...`) & Tipe JSON Inti
```ts
export interface SoilHorizon {
  depth_cm: "0-5" | "5-15" | "15-30" | "30-60" | "60-100" | "100-200";
  clay_pct: number;
  sand_pct: number;
  soc_g_kg: number;
  ph_h2o: number;
  cec_cmol_kg: number;
}

export interface SeasonReplayBundle {
  field_id: string;
  season_id: string;
  crop: "padi" | "jagung";
  variety: string;
  is_simulated: boolean;
  dates: string[];
  rain_mm: number[];
  tmax_c: number[];
  tmin_c: number[];
  et0_mm: number[];
  gdd_cum: number[];
  stage_code: string[];
  ndvi_s2_hls: (number | null)[];
  vh_db_s1: (number | null)[];
  stress_windows: { start: string; end: string; kind: "heat" | "drought" }[];
}
```
