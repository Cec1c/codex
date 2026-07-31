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
