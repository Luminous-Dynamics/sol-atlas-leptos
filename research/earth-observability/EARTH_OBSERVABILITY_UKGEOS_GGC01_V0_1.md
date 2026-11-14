# EARTH-OBSERVABILITY-UKGEOS-GGC01-001
## Multimodal core, wireline and geoscience evidence benchmark candidate v0.1

**Parent:** EARTH-OBSERVABILITY-QUAL-001 (#761)  
**Status:** source metadata and terms review; no underlying data files downloaded or ingested  
**As-of:** 2026-11-13

## 1. Why GGC01 matters

The UKGEOS Glasgow GGC01 Final Borehole Information Pack is unusually valuable for an Earth-observability benchmark because it combines several modalities from one cored, wireline-logged, seismic-monitoring borehole:

- optical whole-core and slabbed-core imagery;
- radiographic core imagery;
- MSCL-S geophysical core scans;
- near-infrared (NIR) scans;
- X-ray fluorescence (XRF) mineralogical and chemical scans;
- core-to-wireline depth integration;
- composite/digital wireline and downhole logs;
- sedimentology, discontinuity and engineering logs;
- drillers' summaries, prognosis, daily drilling records and sample information.

The borehole was drilled between 19 November and 12 December 2018 to 199 m and produced a 102 mm core. Wireline logging took place in December 2018. Five seismometers were installed in February 2019. The final data release includes core scan material acquired in 2019 and in late 2020 / early 2021, and supersedes earlier initial/intermediate releases.

This is an attractive **within-borehole multi-modal integration case**, not a multi-site geological accuracy benchmark.

## 2. Source identity and rights

- Dataset: **UKGEOS Glasgow GGC01 Final Borehole Information Pack**
- DOI: https://doi.org/10.5285/e38c58a6-48ec-4ad1-a996-6c6144968d7d
- Publisher/custodian: British Geological Survey / NERC Environmental Data Service, National Geoscience Data Centre.
- NGDC cited-data record: https://www2.bgs.ac.uk/nationalgeosciencedatacentre/citedData/catalogue/e38c58a6-48ec-4ad1-a996-6c6144968d7d.html
- Accession/download route: https://webapps.bgs.ac.uk/services/ngdc/accessions/index.html#item167252

The exact BGS/NGDC cited-data record states Open Government Licence terms subject to this acknowledgement:

> Contains data supplied by permission of the Natural Environment Research Council [YEAR].

A separately harvested data.gov.uk record still reports “Licence not set”. Keep the discrepancy in the source receipt. For the final pack, preserve the publisher's exact NGDC cited-data record as the terms reference, but verify the accession package manifest, individual files and any embedded third-party notices before admitting each artifact. Do not transfer the final-pack terms to the initial release, intermediate release, separate image packages, other UKGEOS boreholes or unrelated BGS products.

The Open Well Data catalogue can aid discovery, but its catalogue/software licence is separate from the original BGS/NGDC source terms. Do not use its catalogue or code commercially without its own applicable permission.

## 3. Package and data-volume boundary

The final release describes:
- a principal mixed-format archive named UKGEOSGlasgow_GGC01_Final.zip (the source record does not state its total size);
- a separate slabbed high-resolution core image archive of approximately 51 GB;
- a separate previously released intermediate Part Two whole-core optical/radiographic image archive of approximately 71 GB.

Do not download terabytes or start with the large image archives merely because they exist. The first acquisition should be a manifest-only review and a small, rights-qualified text/log subset. Keep each package, release, file and asset separately identified with its own SHA-256, terms, source revision, data dictionary and retrieval receipt.

## 4. Benchmark hypothesis

The first useful experiment is not “predict geology anywhere from one borehole.” It is to establish whether the evidence architecture can align and preserve heterogeneous observations along one known borehole without silently changing depth, units, identity or provenance.

Candidate tasks:

1. **Depth-frame consistency.** Compare core depth, wireline depth, MSCL-S scan depth, NIR/XRF depth, sample intervals and geological log intervals. Record datum, depth reference, run/measurement origin, offset corrections, unit, direction and uncertainty explicitly.
2. **Core-to-wireline registration.** Evaluate depth alignment over contiguous, blind depth blocks. Keep the published core-wireline integration as a source-referenced artifact; do not use it as both fitting input and independent validation truth.
3. **Cross-modal feature correspondence.** Test whether declared boundaries, discontinuities or lithological transitions in one channel coincide with independent changes in another modality at the same depth support.
4. **Sample/interval lineage.** Link core boxes, image strips, scans, lab/sample records and interpreted intervals without assuming a matching sample ID implies identical physical support.
5. **Missingness and detection.** Preserve intervals that were not scanned, not sampled, not recovered, not interpretable or below a sensor's detection capability as distinct states.
6. **Uncertainty and conflicts.** Preserve disagreements between core logs, wireline curves, XRF/NIR outputs and engineering descriptions. Do not automatically select one as authoritative for all targets.
7. **Reproducible provenance.** Bind original files, transformations, units, depth corrections, software versions, calibration records, derived features and benchmark splits by exact identity.

## 5. Split and reference protocol

This source contains one borehole. It cannot support a claim of cross-site, cross-geology or national generalization.

For internal method development:
- split by contiguous depth blocks, not random individual samples;
- use buffer gaps between training and test blocks to reduce adjacent-depth leakage;
- preserve any original acquisition batch, scan pass, core box or logging run as a grouping factor;
- reserve reference measurements from model fitting and hyperparameter selection;
- account for shared processing pipelines and data products derived from the same raw observation;
- report results separately by modality, depth interval, acquisition campaign and target.

Before scoring a physical output, establish which target has independent reference evidence. A published depth-integration product that was constructed from core and wireline inputs is not independent truth for a model that also consumes those inputs. A geological log that informed the interpretation is not independent ground truth for that same interpretation. When no independent reference exists, report a consistency or reproducibility test, not accuracy.

## 6. Matched baselines and comparison

Potential first comparators:
- source-native BGS depth-integrated products;
- direct source-channel alignment with no learned correction;
- linear depth-offset / piecewise registration baselines where assumptions fit;
- expert geologist/geophysicist interpretation, with the reference process recorded;
- modality-removed ablations (core only, wireline only, MSCL-S only, NIR/XRF only);
- uncertainty-aware multi-modal alignment only after independent reference design.

Do not compare GGC01 results directly with a multi-site learned geological model or claim superiority over Leapfrog, OpenGround, geophysical inversion packages or commercial core-scanning systems. Their targets, evidence support, output semantics and validation populations differ.

## 7. Admission gates before any download

- Verify the exact NGDC accession and final-pack version match the cited-data DOI.
- Capture the exact package manifest and names/sizes of all files.
- Reconcile the OGL acknowledgement and data.gov.uk's blank licence field; preserve both records.
- Inspect per-file and embedded notices; separate third-party or restricted assets.
- Confirm data dictionaries, abbreviations, units, depth datums, vertical references, sample IDs and interval semantics.
- Capture sensor/instrument, processing, calibration and acquisition metadata.
- Decide which files are needed for the first bounded task; avoid the 51 GB and 71 GB image packages until required.
- Keep raw data in source-preserving, content-addressed storage; no source file is rewritten.
- Do not place downloaded BGS assets in public CI or publish derivatives until their permission and disclosure conditions are explicit.

## 8. Proposed first implementation slice

1. Add a manifest-only source inventory from the publisher's accession record.
2. Parse AGS 4.x files using the local structural preflight, but obtain the official BGS AGS and BGS/NGDC checks for each exact file using the explicitly declared dictionary.
3. Preserve LAS/DLIS metadata, depth curves, units, run IDs and processing history without silently converting data.
4. Define an explicit depth-support object: borehole ID, depth reference, interval bounds, unit, measurement support, acquisition run, time, transform/correction and uncertainty.
5. Add round-trip and cross-reference tests between AGS, LAS/DLIS, sample/interval metadata and image/scan indices.
6. Only then build a within-borehole depth-alignment benchmark with contiguous held-out blocks and independent reference channels.

No data file has been downloaded or ingested in this workstream. The profile is source discovery and benchmark design only.

## 9. References

- BGS/NGDC GGC01 Final Borehole Information Pack: https://www2.bgs.ac.uk/nationalgeosciencedatacentre/citedData/catalogue/e38c58a6-48ec-4ad1-a996-6c6144968d7d.html
- data.gov.uk harvested metadata: https://www.data.gov.uk/dataset/cac45dbb-eaf3-4378-bd7c-5f890dc2c218/ukgeos-glasgow-ggc01-final-borehole-information-pack
- NERC report describing the final release: https://nora.nerc.ac.uk/id/eprint/530762/
- BGS NGDC access-data guidance: https://www.bgs.ac.uk/ngdc/access-data/
