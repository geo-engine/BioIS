# Climate risk: query precomputed yearly indicator datasets instead of raw daily data

## Problem

For every request, `climate-risk` builds this GeoEngine workflow over the raw daily NEX-GDDP-CMIP6 files on S3:

`MdGdalSource → Expression → TemporalRasterAggregation (yearly sum) → RasterVectorJoin`

These files store **one compressed block per day for the whole globe** (`[1, 600, 1440]`, DEFLATE, about 244 MB per year file). To read one point for one day, GDAL must download that day's global block. A query over N years therefore downloads N whole files.

- **Live test** (1 model, heat + ice days, 5-year window + 5-year reference): **803 s**, about 4.9 GB downloaded at about 6 MB/s.
- **Default request** (5 models × 6 indicators × (3 scenarios × 30 + 30 reference) years ≈ 3,600 year files): **about 880 GB, about 40 h**. HTTP times out long before that.
- GeoEngine's tile cache and GDAL's block cache avoid repeated downloads within one run. They don't make a new location or time window cheap, and 880 GB doesn't fit in any cache.

## Proposal

Compute the yearly event-day counts once and store them as GeoEngine datasets. The backend queries those.

1. **Ingest** the raw daily datasets (existing notebooks, unchanged).
2. **Aggregate** in a new notebook `notebooks/nexgddp_cmip6_aggregate_yearly.ipynb`: for each model × scenario × indicator, register the same yearly-sum workflow the backend builds today. Then call `datasetFromProcessingGraph` with the global extent, the scenario's full year range and `asCog: true`. GeoEngine writes one COG per year as a `GdalSource` dataset.
   - **One dataset per indicator × model × scenario,** each a single-band time series. GeoEngine cannot materialize multi-band results (`raster_stream_to_geotiff` rejects queries with several bands), so heat and ice days cannot share one dataset.
3. **Query** from the backend: `GdalSource(nexgddp_cmip6_{model}_{scenario}_{indicator}_yearly) → RasterVectorJoin`.

The aggregation, probability and anomaly code in the backend stays unchanged, because the per-year values are the same numbers.

## Sizes

Measured on ACCESS-CM2, SSP2-4.5, 2050: one count grid of 1440 × 600 cells, 16-bit integers, 512-pixel blocks, DEFLATE.

| Indicator | Range on land | Size per year |
|---|---|---|
| heatDays | 0–365 | 0.28 MB |
| iceDays | 0–365 | 0.19 MB |
| dryDays | 14–365 | 0.31 MB |
| heavyRainDays | 0–170 | 0.16 MB |

- Only 30.7 % of the cells are land; the ocean is masked. Counts reach 365, so 8-bit integers are not enough.
- **As GeoEngine stores it** (LZW + internal mask), heat days 1950 take **0.31 MB** per year, about the same as DEFLATE.
- **Total:** 6 indicators × 5 models × 323 years (65 historical + 3 × 86) = **9,690 yearly grids in 120 datasets, about 2.5–3 GB**.

## Build time (measured on the local dev GeoEngine, a debug build)

| Run | Time | Downloaded |
|---|---|---|
| heatDays, one fresh year | 265 s | 314 MB |
| iceDays, same `tasmax` year right after | 127 s | 1 MB |

- **The cache works.** Each raw year file is downloaded about once (314 MB for a 244 MB file), not once per tile, and the second indicator of the same source reads from the cache. The notebook orders targets so pairs run back to back.
- **The computation dominates.** About 127 s per indicator-year in the **debug build**: 9,690 indicator-years would take about 340 h one at a time. The full build must run on a **release build** of GeoEngine, with several tasks in parallel (`MAX_PARALLEL_TASKS`).
- **Download:** about 4,850 raw year files × ~310 MB ≈ **1.5 TB** once.

### With the new GDAL writer (release build, tile cache on)

| Run (ACCESS-CM2) | Time | Downloaded |
|---|---|---|
| heatDays 1950 | 2 min 30 s | 594 MB |
| iceDays 1950, right after | 3 s | 0.04 MB |
| heatDays historical 1950–1954 | 8.7 min | |
| iceDays historical 1950–1954 | 0.3 min | |
| heatDays SSP2-4.5 2015–2019 | 10.3 min | |
| iceDays SSP2-4.5 2015–2019 | 0.5 min | |

- With `[cache] enabled`, `default_ttl_seconds > 0` and one task at a time, the second indicator of a raw variable reads it entirely from the cache. Each raw year is downloaded once per variable.
- About 2 min per raw variable-year: 5 models × 3 variables × 323 years ≈ 4,850 variable-years ≈ 160 h one at a time. Parallel tasks on different raw variables would shorten that; untested.
- The writer stores 0.19–0.26 MB per indicator-year (ZSTD, internal mask), plus a `dataset.json` with definition and tiles that the export notebook packs.
- **Live backend query** (ACCESS-CM2, heat + ice days, 2015–2019 against 1950–1954): **0.10 s**, compared with 803 s on the raw daily data.

**A query afterwards:** one small block (about 50 KB) per year per dataset, read from local disk. A default request reads about 3,600 of them (about 180 MB): seconds instead of hours. This is an estimate, not yet measured.

## What changes

- **New aggregation notebook.** It also writes `backend/conf/nexgddp_models.toml`, and lists a model/scenario only when all 6 of its yearly datasets exist.
- **Backend `workflow.rs`:** one `GdalSource` per (model, scenario, indicator). The `Expression` and `TemporalRasterAggregation` steps and the threshold expressions leave the Rust code.
- **Raw daily datasets** stay registered, but are only needed for rebuilding.

## Transfer from system A to system B

The yearly datasets are computed on system A, which holds the raw datasets and the workflow. BioIS runs against system B, so the datasets are moved after the build. Two generic notebooks handle this:

- **Export** (`notebooks/geoengine_dataset_export.ipynb`): for each dataset (selected by the tag `nexgddp-yearly`), read its properties and its file list (loading info) from A's API. Copy its yearly files from A's storage folder and write `dataset.json`, which is exactly the definition GeoEngine needs to re-create the dataset, with the file paths made relative. A `manifest.json` records A's version, a SHA-256 checksum for every file, and the spot-check values.
- **Move** the export folder into a GeoEngine volume on B (a folder configured in GeoEngine), for example with `rsync`.
- **Import** (`notebooks/geoengine_dataset_import.ipynb`, admin on B): check all checksums before writing anything. Create each dataset in the volume under the same name, which also grants read access to all users. Then re-run the spot checks on B and confirm that every dataset the BioIS model registry needs exists.

Because dataset names stay the same, the backend needs no configuration change between A and B.

**Tested:** a round trip on the local GeoEngine (export, copy into the `dev_data` volume, import under a test suffix). Checksums matched, the spot-check values were equal, and tags, provenance, symbology and anonymous read access came through.

## Open questions for review

1. **Where the thresholds live.** Should the notebook (Python) be the only place that defines them (≥ 30 °C and so on)? The alternative is a `biois-cli` command that prints the workflow from the Rust code.
2. **Build machine.** The build needs a release build of GeoEngine near the data (ideally AWS `us-west-2`). Which machine is system A for the full build, and how many tasks can it run in parallel?
3. **GFDL-ESM4 (365-day calendar).** Yearly sums follow the raw dataset's Gregorian labels, so leap years miss Dec 31 (at most 1 day of bias). Accept, or handle it?
4. **Rebuilds.** GeoEngine refuses dataset names that already exist, so a rebuild must delete the old dataset first. Who triggers rebuilds (new model, corrected files)?

## Later: how GeoEngine writes the datasets

To be investigated in the GeoEngine backend before the full build:

- **Storage.** `datasetFromProcessingGraph` writes into GeoEngine's upload directory on its volume (≤ 5 GB). Is that the right place, and is it backed up?
- **No data.** The writer sets `no_data_value: None` and stores an internal mask instead. A first check shows no data at an ocean point; confirm this for all indicators.
- **Format.** The writer uses LZW level 9 and 512-pixel blocks (0.31 MB per year measured). Is the read speed of the point query acceptable?
- **Tag filter.** `GET /datasets?tags=…` rejects a plain value and ignores the bracket forms, so the export notebook filters tags itself.
- **Failed tasks.** The task deletes its files when it fails. Can a failure leave a dataset entry behind (the code has a TODO for this)?

## Prior design

<link>, dropped because <reason>. What is different now: <…>.

## Acceptance criteria

- 120 yearly datasets registered on B, readable by anonymous and registered users.
- Export → import round trip: checksums match, and the spot-check values on B equal those on A.
- Spot check: the precomputed value equals the on-the-fly value for 3 land points × 2 years × 2 indicators.
- A default climate-risk request on dev answers in under 30 s (proposed target).
- Backend tests and the live GeoEngine test pass.
