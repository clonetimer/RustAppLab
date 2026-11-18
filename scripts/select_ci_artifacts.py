#!/usr/bin/env python3
"""Select the newest upload for each artifact name in this workflow run.

Artifact IDs are not chronological. Selection uses creation timestamps only;
the assembler still decides whether the selected evidence passes its gates.
"""
from __future__ import annotations
import argparse
from datetime import datetime
import json
import os
import re
import sys
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def artifact_id(artifact: dict) -> int:
    if not isinstance(artifact, dict):
        raise ValueError('Artifact metadata must be an object.')
    identifier = artifact.get('id')
    if type(identifier) is not int or identifier <= 0:
        raise ValueError('Artifact ID must be a positive integer.')
    return identifier


def select_artifact_ids(artifacts: list[dict]) -> list[int]:
    """Fail closed on incomplete/ambiguous metadata; never fall back to a PASS."""
    if not isinstance(artifacts, list) or not artifacts:
        raise ValueError('No artifact list is available for selection.')
    groups = {}
    seen_ids = set()
    for artifact in artifacts:
        identifier = artifact_id(artifact)
        if identifier in seen_ids:
            raise ValueError('Duplicate artifact ID in this workflow run.')
        seen_ids.add(identifier)
        name = artifact.get('name')
        if not isinstance(name, str) or not name.strip():
            raise ValueError('Artifact name must be a nonempty string.')
        created_at = artifact.get('created_at')
        if not isinstance(created_at, str):
            raise ValueError('Artifact creation time must be an aware ISO 8601 timestamp.')
        try:
            created = datetime.fromisoformat(created_at.replace('Z', '+00:00'))
        except ValueError:
            raise ValueError('Artifact creation time is not valid ISO 8601.') from None
        if created.tzinfo is None or created.utcoffset() is None:
            raise ValueError('Artifact creation time must include a timezone.')
        expired = artifact.get('expired')
        if type(expired) is not bool:
            raise ValueError('Artifact expiration state must be a boolean.')
        groups.setdefault(name, []).append((created, identifier, expired))

    selected = []
    for name in sorted(groups):
        newest = max(item[0] for item in groups[name])
        candidates = [item for item in groups[name] if item[0] == newest]
        if len(candidates) != 1:
            raise ValueError('Artifact name has ambiguous newest creation timestamps.')
        _, identifier, expired = candidates[0]
        if expired:
            raise ValueError(f'Newest artifact {identifier} is expired; refusing fallback.')
        selected.append(identifier)
    return selected


def repository_name(value: str) -> str:
    if (not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', value)
            or any(part in ('.', '..') for part in value.split('/'))):
        raise argparse.ArgumentTypeError('Repository must be owner/repo.')
    return value


def positive_run_id(value: str) -> int:
    if not re.fullmatch(r'[0-9]+', value) or int(value) <= 0:
        raise argparse.ArgumentTypeError('Run ID must be a positive integer.')
    return int(value)


def fetch_run_artifacts(repository: str, run_id: int, token: str) -> list[dict]:
    """List only this run; reject partial, changing, or repeated API pages."""
    artifacts = []
    seen_ids = set()
    total = None
    page = 1
    while total is None or len(artifacts) < total:
        request = Request(
            f'https://api.github.com/repos/{repository}/actions/runs/{run_id}/artifacts?per_page=100&page={page}',
            headers={'Authorization': 'Bearer ' + token,
                     'Accept': 'application/vnd.github+json',
                     'X-GitHub-Api-Version': '2022-11-28'})
        try:
            with urlopen(request, timeout=30) as response:
                data = json.load(response)
        except HTTPError as error:
            # Never print server bodies, request headers, or upstream error text.
            raise ValueError(f'GitHub artifact API returned HTTP {error.code}.') from None
        except (OSError, ValueError):
            raise ValueError('GitHub artifact API request failed or returned invalid JSON.') from None
        if not isinstance(data, dict):
            raise ValueError('GitHub artifact API response must be an object.')
        count, items = data.get('total_count'), data.get('artifacts')
        if type(count) is not int or count < 0 or not isinstance(items, list):
            raise ValueError('GitHub artifact API response has invalid count or artifacts.')
        if total is not None and count != total:
            raise ValueError('Artifact count changed during pagination; refusing partial evidence.')
        total = count
        if len(items) > 100 or len(artifacts) + len(items) > total:
            raise ValueError('Artifact pagination exceeds its reported count.')
        for artifact in items:
            identifier = artifact_id(artifact)
            if identifier in seen_ids:
                raise ValueError('Artifact pagination repeated an ID.')
            seen_ids.add(identifier)
        artifacts.extend(items)
        if len(artifacts) < total and len(items) != 100:
            raise ValueError('Artifact pagination ended before its reported count.')
        page += 1
    return artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=repository_name, required=True)
    parser.add_argument('--run-id', type=positive_run_id, required=True)
    arguments = parser.parse_args(argv)
    try:
        token = os.environ.get('GH_TOKEN', '')
        if not token.strip():
            raise ValueError('GH_TOKEN is required to list workflow artifacts.')
        artifacts = fetch_run_artifacts(arguments.repository, arguments.run_id, token)
        # A rerun may retain the consumer's own previous output. It is not input
        # evidence, and including it would recursively collect old candidates.
        artifacts = [item for item in artifacts if item.get('name') != 'candidate-release']
        selected = select_artifact_ids(artifacts)
        by_id = {item['id']: item for item in artifacts}
        result = {'schema': 1, 'repository': arguments.repository, 'run_id': arguments.run_id,
                  'artifacts': [by_id[identifier] for identifier in selected]}
        print(json.dumps(result, sort_keys=True))
        return 0
    except ValueError as error:
        print('Artifact selection refused: ' + str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
