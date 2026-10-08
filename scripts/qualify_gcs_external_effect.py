        raise AssertionError("wrong observer isolation audit schema")
    observer = wif_verification.get("observer_service_account")
    effect = wif_verification.get("service_account")
    if audit.get("observer_service_account") != observer:
        raise AssertionError("observer isolation observer identity drift")
    if audit.get("effect_service_account") != effect:
        raise AssertionError("observer isolation effect identity drift")
    if not isinstance(observer, str) or not observer:
        raise AssertionError("WIF observer identity is missing")
    expected_principal = "serviceAccount:" + observer
    if audit.get("observer_principal") != expected_principal:
        raise AssertionError("observer isolation principal drift")
    if audit.get("required_observer_permission_verified") is not True:
        raise AssertionError("observer IAM read privilege was not verified")
    if audit.get("forbidden_effect_permissions_absent") is not True:
        raise AssertionError("observer effect privileges were not excluded")
    observed = audit.get("observed_permissions")
    if not isinstance(observed, list):
        raise AssertionError("observer isolation permissions are malformed")
    if "iam.serviceAccounts.getIamPolicy" not in observed:
        raise AssertionError("observer IAM policy-read permission is missing")
    forbidden = audit.get("forbidden_permissions")
    if not isinstance(forbidden, list):
        raise AssertionError("observer forbidden permission list is malformed")
    if set(observed).intersection(
        str(permission) for permission in forbidden
    ):
        raise AssertionError("observer isolation contains forbidden permissions")
    digest_value = audit.get("policy_analyzer_response_digest")
    if not is_sha256_digest(digest_value):
        raise AssertionError("observer isolation response digest is malformed")
    if audit.get("project_pivot_permissions") != [
        "cloudbuild.builds.create",
        "deploymentmanager.deployments.create",
        "compute.instances.create",
        "run.services.create",
        "run.jobs.create",
        "cloudfunctions.functions.create",
        "resourcemanager.projects.setIamPolicy",
    ]:
        raise AssertionError("observer isolation pivot permission set drift")
    if audit.get("project_pivot_permissions_absent") is not True:
        raise AssertionError(
            "observer isolation project pivots were not excluded"
        )
    if audit.get("project_pivot_observed_permissions") != []:
        raise AssertionError(
            "observer isolation contains project pivot permissions"
        )
    if not is_sha256_digest(audit.get("project_pivot_response_digest")):
        raise AssertionError(
            "observer isolation project pivot digest is malformed"
        )
    expected_observer_profile_permissions = observer_profile.get(
        "forbidden_project_permissions"
    )
    if audit.get("observer_authority_permissions") != (
        expected_observer_profile_permissions
    ):
        raise AssertionError(
            "observer authority permission set drift"
        )
    if audit.get("observer_authority_permissions_absent") is not True:
        raise AssertionError(
            "observer authority permissions were not excluded"
        )
    if audit.get("observer_authority_observed_permissions") != []:
        raise AssertionError(
            "observer authority findings were present"
        )
    if not is_sha256_digest(
        audit.get("observer_authority_response_digest")
    ):
        raise AssertionError(
            "observer authority response digest is malformed"
        )
    lateral_permissions = observer_profile.get(
        "forbidden_service_account_permissions"
    )
    if audit.get("lateral_service_account_permissions") != (
        lateral_permissions
    ):
        raise AssertionError(
            "lateral observer permission set drift"
        )
    if audit.get("lateral_service_account_permissions_absent") is not True:
        raise AssertionError(
            "lateral service-account permissions were not excluded"
        )
    if audit.get("lateral_service_account_findings") != []:
        raise AssertionError(
            "lateral service-account findings were present"
        )
    if not is_sha256_digest(
        audit.get("lateral_service_account_response_digest")
    ):
        raise AssertionError(
            "lateral service-account response digest is malformed"
        )
    claim = audit.get("claim_ceiling")
    if not isinstance(claim, str) or not claim:
        raise AssertionError("observer isolation claim ceiling is missing")


def validate_broad_sa_audit(
    audit: dict[str, object],
    wif_verification: dict[str, object],
    observer_oidc_claims: dict[str, object],
) -> None:
    if audit.get("schema") != BROAD_SA_AUDIT_SCHEMA:
        raise AssertionError("wrong broad service-account audit schema")
    if audit.get("wif_verification_digest") != digest(wif_verification):
        raise AssertionError("broad-SA WIF evidence digest drift")
    if audit.get("wif_profile_path") != WIF_TRUST_PROFILE_PATH:
        raise AssertionError("broad-SA profile path drift")
    if audit.get("wif_profile_digest") != wif_verification.get(
        "profile_digest"
    ):
        raise AssertionError("broad-SA profile digest drift")
    if audit.get("service_account") != wif_verification.get(
        "service_account"
    ):
        raise AssertionError("broad-SA service-account drift")
    if audit.get("observer_service_account") != wif_verification.get(