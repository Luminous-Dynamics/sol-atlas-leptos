            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.recompute_manifest_hash().unwrap();
        assert_eq!(
            frontier.validate_temporal_manifest_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn strict_manifest_rejects_late_source_metadata() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1951),
            capture_time: None,
            available_by: 1951,
        }];
        frontier.recompute_manifest_hash().unwrap();
        assert_eq!(
            frontier.validate_temporal_manifest_strict(),
            Err(ProjectionError::LaterEvidenceInFrontier)
        );
    }

    #[test]
    fn strict_manifest_accepts_complete_source_metadata() {
        let mut frontier = frontier();
        frontier.evidence_metadata = vec![
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:old".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1940),
                capture_time: None,
                available_by: 1940,
                validity_time: None,
            },
            EvidenceTemporalMetadataV1 {
                evidence_id: "e:transition".into(),
                source_snapshot: "source:archive".into(),
                artifact_time: None,
                publication_time: Some(1945),
                capture_time: None,
                available_by: 1945,
                validity_time: None,
            },
        ];
        frontier.source_metadata = vec![SourceSnapshotTemporalMetadataV1 {
            source_snapshot: "source:archive".into(),
            publication_time: Some(1940),
            capture_time: None,
            available_by: 1940,
        }];
        frontier.recompute_manifest_hash().unwrap();
        assert_eq!(frontier.validate_temporal_manifest_strict(), Ok(()));
    }

    #[test]
    fn strict_chain_rejects_incomplete_source_metadata() {
        let mut root = frontier();
        root.evidence_metadata = vec![EvidenceTemporalMetadataV1 {
            evidence_id: "e:old".into(),
            source_snapshot: "source:archive".into(),
            artifact_time: None,
            publication_time: Some(1939),
            capture_time: None,
            available_by: 1939,
            validity_time: None,
        }];
        root.recompute_manifest_hash().unwrap();
        assert_eq!(
            (EvidenceFrontierChainV1 {
                frontiers: vec![root]
            })
            .validate_strict(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn replay_filters_by_epoch_and_frontier() {
        let request = TemporalProjectionRequestV1 {
            map_epoch: YearInterval {
                from: Some(1940),
                to: Some(1950),
            },
            evidence_frontier: frontier(),
        };
        let snapshots = vec![
            snapshot(
                "snapshot:b",
                YearInterval {
                    from: Some(1945),
                    to: Some(1947),
                },
                "frontier:1949",
                "e:old",
            ),
            snapshot(
                "snapshot:a",
                YearInterval {
                    from: Some(1930),
                    to: Some(1939),
                },
                "frontier:1949",