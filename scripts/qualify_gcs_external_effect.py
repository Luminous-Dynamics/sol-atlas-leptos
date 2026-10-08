    )
    broad_sa_audit = json.loads(
        Path(broad_sa_audit_path).read_text(encoding="utf-8")
    )
    observer_isolation_audit = json.loads(
        Path(observer_isolation_audit_path).read_text(encoding="utf-8")
    )
    policy_effect_audit = json.loads(
        Path(policy_effect_audit_path).read_text(encoding="utf-8")
    )
    validate_effective_iam_audit(
        effective_iam_audit,
        wif_verification,
        observer_oidc_claims,
    )
    validate_broad_sa_audit(
        broad_sa_audit,
        wif_verification,
        observer_oidc_claims,
    )
    validate_observer_isolation_audit(
        observer_isolation_audit,
        wif_verification,
    )
    run_id = os.environ.get("GITHUB_RUN_ID", "local")
    run_attempt = os.environ.get("GITHUB_RUN_ATTEMPT", "1")
    expected_object_root = (
        object_prefix.rstrip("/")
        + f"/run-{run_id}-attempt-{run_attempt}"
    )
    if object_root is not None and object_root != expected_object_root:
        raise AssertionError("qualification object root is not bound to this run")
    root = object_root or (
        expected_object_root + "-" + uuid.uuid4().hex[:12]
    )
    main_name = root + "/main.bin"
    point_name = root + "/point-in-time.bin"
    race_name = root + "/metadata-race.bin"
    validate_policy_effect_audit(
        policy_effect_audit,
        wif_verification,
        bucket,
        [main_name, point_name, race_name],
        str(wif_verification.get("project_id", "")),
    )
    oidc_audience = wif_verification.get("oidc_expected_audience")
    if not isinstance(oidc_audience, str) or not oidc_audience:
        raise AssertionError("WIF verification has no OIDC expected audience")
    observed_claims = oidc_claims.get("claims")
    if not isinstance(observed_claims, dict):
        raise AssertionError("missing observed GitHub OIDC claims")
    if oidc_claims.get("audience") != observed_claims.get("aud"):
        raise AssertionError("OIDC artifact audience does not match its claims")
    verify_oidc_claim_identity(observed_claims, github_context, oidc_audience)
    wif_credential_config_verification = (
        load_wif_credential_config_verification(
            wif_credential_config_verification_path,
            wif_verification,
            oidc_claims,
        )
    )
    source_commit = verify_checked_out_source_commit()
    token = access_token()
    run_id = os.environ.get("GITHUB_RUN_ID", "local")
    run_attempt = os.environ.get("GITHUB_RUN_ATTEMPT", "1")
    race_name = root + "/metadata-race.bin"
    resource = GcsGenerationFencedObject(bucket, main_name, token)
    point_resource = GcsGenerationFencedObject(bucket, point_name, token)
    race_resource = GcsGenerationFencedObject(bucket, race_name, token)
    cases: list[dict[str, object]] = []

    try:
        generation = create_setup(resource, "main")
