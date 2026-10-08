    try:
        module.verify_resource_identity(report)
    except AssertionError:
        return
    raise AssertionError(message)


def main() -> None:
    assert module.SCHEMA == (
        "sol-atlas:recovery-execution-effect-external-report:v7"
    )
    assert module.EFFECTIVE_IAM_AUDIT_SCHEMA == (
        "sol-atlas:gcs-wif-effective-iam-audit:v8"
    )
    assert module.EFFECTIVE_IAM_PROJECT_PIVOT_PERMISSIONS == [
        "cloudbuild.builds.create",
        "deploymentmanager.deployments.create",
        "compute.instances.create",
        "run.services.create",
        "run.jobs.create",
        "cloudfunctions.functions.create",
        "resourcemanager.projects.setIamPolicy",
    ]
    good = {
        "service": "Google Cloud Storage",
        "bucket": "sol-atlas-qualification",
        "object_prefix": PREFIX,
        "object_names": [
            PREFIX + "/main.bin",
            PREFIX + "/point-in-time.bin",
            PREFIX + "/metadata-race.bin",
        ],
    }
    module.verify_resource_identity(good)

    server_good = {
        "schema": module.GITHUB_RUN_VERIFICATION_SCHEMA,
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "repository_owner_id": "216969177",
        "workflow_name": "Qualify GCS external effect",
        "workflow_path": ".github/workflows/qualify-gcs.yml",
        "workflow_id_frozen": 311325850,