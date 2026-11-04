# HDB-20: Rights-Aware Dataset Intake v0.1

## Purpose

HDB-20 stops dataset availability, paper citations and software licences from being conflated with permissions for imagery, point clouds, meshes, annotations, model weights and derivative assets.

## Implemented

- Versioned package and per-asset registry.
- Separate source-code/data/annotation/model assets.
- Source URI, version, citation, availability, per-asset file digest and byte size.
- Separate asset licence identifier/URI; a code licence is not inherited by data.
- Purpose-specific permissions for local inspection, research training, evaluation, derivative storage, publication, redistribution and commercial use.
- Explicit granted/denied/unknown permissions; granted requires evidence reference, denied requires a condition/reason.
- Unknown permission blocks that purpose. Publicly downloadable availability never upgrades permission.
- Access tier and coordinate sensitivity; exact sensitive coordinates cannot be declared public and non-public access tiers require an access receipt.
- Denied-use decisions require both a restriction summary and a rights-evidence reference. Package archive digest and byte size are recorded together.
- Parent asset references and cycle checking.
- Deterministic use decision with claim ceiling limited to recorded permission presence; legal document authenticity and legal interpretation remain external.

## Known dataset candidates

### Archaeological stone/wall model
Uziel, Zohar & Jaffe (2026), JAS 185, 106430, DOI 10.1016/j.jas.2025.106430. The article says its data/code are openly available in Zenodo 10.5281/zenodo.15673668 and the reproducibility editor reproduced the results. Package-level licence and precise asset inventory still require direct verification. Keep all use-purpose permissions unknown until the source deposit's licence/terms and files are inspected.

### Three-leaf rubble stone masonry scans
Saloustros et al. (2023), Scientific Data 10, 533, DOI 10.1038/s41597-023-02417-3; Zenodo 10.5281/zenodo.7093710. The deposit lists 2.5 GB raw, 6.0 GB processed and 36.2 kB analysis archive plus README. It describes individual stone point clouds/meshes and wall assembly data. The currently rendered record page does not show a populated licence field; do not presume reuse rights.

### Image-based stone masonry digital twins
Pantoja-Rosero et al. (2023), Automation in Construction 145, 104632, DOI 10.1016/j.autcon.2022.104632; Zenodo 10.5281/zenodo.7266587. The deposit lists a 36.1 GB synthetic package and README. Its rendered licence field is blank. The related source-code repository advertises GPL-3.0, but that software licence does not automatically grant rights to the separately hosted data.

### Shivta 2026
Uziel et al. (2026), PNAS, DOI 10.1073/pnas.2628851123. Use the published results for paper comparison. Do not assume raw images, labels, weights or custodial coordinates are released under reusable rights without asset-level evidence.

## Limits

A valid registry only validates the recorded structure and the existence of evidence references. It does not authenticate the licence, source depositor, signature, permissions, custodian, or whether a legal interpretation is correct. Human rights review remains mandatory for uncertain or sensitive material.

## Dataset intake order

1. Uziel 9-site release — exact paper reproduction after asset and rights verification.
2. EPFL three-wall scans — auxiliary metric geometry, stone identity and assembly evaluation after rights verification.
3. EPFL synthetic image-based twins — reconstruction/registration testing after rights verification.
4. Shivta raw imagery/labels — request author/custodial access if no explicit reusable release is available.

## Research references

- Uziel et al. (2026): https://doi.org/10.1016/j.jas.2025.106430
- Saloustros et al. (2023): https://doi.org/10.1038/s41597-023-02417-3
- Pantoja-Rosero et al. (2023): https://doi.org/10.1016/j.autcon.2022.104632
- EPFL EESD datasets: https://www.epfl.ch/labs/eesd/data_sets/
