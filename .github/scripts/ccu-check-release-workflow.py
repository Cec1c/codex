from pathlib import Path


WORKFLOW = Path(__file__).parents[1] / "workflows" / "ccu-i18n-release.yml"
RESOLVER_WORKFLOW = (
    Path(__file__).parents[1] / "workflows" / "ccu-conflict-resolver.md"
)


def main() -> None:
    workflow = WORKFLOW.read_text(encoding="utf-8")
    ordered_markers = [
        "gh issue create --repo",
        'remote_base_commit="$(',
        'if [[ -z "$remote_base_commit" ]] && ! git push origin',
        "gh workflow run ccu-conflict-resolver.lock.yml",
    ]
    positions = [workflow.index(marker) for marker in ordered_markers]
    if positions != sorted(positions):
        raise SystemExit(
            "conflict handoff must record the issue before creating the agent base "
            "branch and dispatching the resolver"
        )

    prepared_ref_markers = [
        'remote_prepared_commit="$(',
        'git ls-remote origin "refs/heads/$release_branch"',
        'if [[ -z "$PREPARED_REF" && -n "$remote_prepared_commit" ]]',
        'PREPARED_REF="$release_branch"',
        'if [[ -n "$PREPARED_REF" ]]',
        'git fetch --no-tags origin "$PATCH_REF:refs/remotes/origin/ccu-patch-source"',
    ]
    prepared_ref_positions = [
        workflow.index(marker) for marker in prepared_ref_markers
    ]
    if prepared_ref_positions != sorted(prepared_ref_positions):
        raise SystemExit(
            "existing prepared release branches must be detected and validated before "
            "replaying the historical patch stack"
        )

    duplicate_conflict_markers = [
        'conflict_fingerprint="$(',
        'echo "<!-- ccu-sync-conflict-fingerprint:$conflict_fingerprint -->"',
        "unchanged_conflict=false",
        'gh issue view "$issue_number"',
        'if grep -Fq --',
        'unchanged_conflict=true',
        'else\n                  gh issue edit "$issue_number"',
        'if [[ "$GITHUB_EVENT_NAME" == "schedule"',
        "skipping duplicate resolver dispatch",
        "gh workflow run ccu-conflict-resolver.lock.yml",
    ]
    duplicate_conflict_positions = [
        workflow.index(marker) for marker in duplicate_conflict_markers
    ]
    if duplicate_conflict_positions != sorted(duplicate_conflict_positions):
        raise SystemExit(
            "scheduled release retries must suppress unchanged conflict dispatches "
            "without blocking explicit workflow retries"
        )

    fingerprint_start = workflow.index('conflict_fingerprint="$(')
    fingerprint_end = workflow.index('\n              )"', fingerprint_start)
    fingerprint_block = workflow[fingerprint_start:fingerprint_end]
    if "upstream_main_commit" in fingerprint_block:
        raise SystemExit(
            "conflict deduplication must ignore the moving upstream main snapshot"
        )
    if 'grep -Fq -- "<!-- ccu-sync-metadata:$metadata -->"' in workflow:
        raise SystemExit(
            "conflict deduplication must not compare the full diagnostic metadata"
        )

    required_markers = [
        'gh issue comment "$issue_number"',
        "The workflow token could not create",
        "Create the branch with trusted maintainer credentials",
    ]
    missing = [marker for marker in required_markers if marker not in workflow]
    if missing:
        raise SystemExit(f"release workflow is missing recovery markers: {missing}")

    resolver_workflow = RESOLVER_WORKFLOW.read_text(encoding="utf-8")
    if "safe-outputs:\n  report-failure-as-issue: false\n" not in resolver_workflow:
        raise SystemExit(
            "the conflict resolver must not create redundant workflow failure issues"
        )
    resolver_dispatch_inputs = [
        "   - `upstream_tag = metadata.upstream_tag`",
        "   - `revision = metadata.revision` 的字符串形式",
        "   - `patch_ref = metadata.patch_ref`",
        "   - `prepared_ref = metadata.release_branch`",
        "   - `alpha = false`",
        '   - `alpha_sequence = "1"`',
    ]
    missing_dispatch_inputs = [
        marker for marker in resolver_dispatch_inputs if marker not in resolver_workflow
    ]
    if missing_dispatch_inputs:
        raise SystemExit(
            "resolver dispatch contract is missing required inputs: "
            f"{missing_dispatch_inputs}"
        )


if __name__ == "__main__":
    main()
