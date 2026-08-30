from pathlib import Path


WORKFLOW = Path(__file__).parents[1] / "workflows" / "ccu-i18n-release.yml"


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
        "unchanged_conflict=false",
        'gh issue view "$issue_number"',
        'grep -Fq -- "<!-- ccu-sync-metadata:$metadata -->"',
        'unchanged_conflict=true',
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

    required_markers = [
        'gh issue comment "$issue_number"',
        "The workflow token could not create",
        "Create the branch with trusted maintainer credentials",
    ]
    missing = [marker for marker in required_markers if marker not in workflow]
    if missing:
        raise SystemExit(f"release workflow is missing recovery markers: {missing}")


if __name__ == "__main__":
    main()
