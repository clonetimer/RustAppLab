#!/usr/bin/env python3
"""Create an immutable lightweight checkpoint after successful main-push CI.

Uses the checkout's normal Git credentials. Does not discover tokens, force tags,
create releases, or change remote branches.
"""
from __future__ import annotations
import argparse
import os
from pathlib import Path
import re
import subprocess
import sys


def git(repository: Path, *arguments: str) -> str:
    result = subprocess.run(['git', '-C', str(repository), *arguments], check=True,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return result.stdout.strip()


def checkpoint(repository: Path) -> str:
    if os.environ.get('GITHUB_EVENT_NAME') != 'push' or os.environ.get('GITHUB_REF') != 'refs/heads/main':
        raise ValueError('Checkpoint tags require a push event on refs/heads/main.')
    sha = os.environ.get('GITHUB_SHA', '')
    if not re.fullmatch(r'[0-9a-fA-F]{40}', sha):
        raise ValueError('GITHUB_SHA must be a complete 40-hex commit SHA.')
    sha = sha.lower()
    if git(repository, 'rev-parse', 'HEAD') != sha:
        raise ValueError('HEAD does not exactly match GITHUB_SHA.')
    version = (repository / 'VERSION').read_text(encoding='utf-8').strip()
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z][0-9A-Za-z.-]*)?', version):
        raise ValueError('VERSION is not safe for a checkpoint tag.')
    tag = f'checkpoint/v{version}-{sha[:12]}'
    reference = 'refs/tags/' + tag
    git(repository, 'check-ref-format', reference)
    existing = git(repository, 'ls-remote', '--refs', 'origin', reference)
    if existing:
        rows = [line.split() for line in existing.splitlines()]
        if rows == [[sha, reference]]:
            return f'Checkpoint {tag} already points to {sha}; unchanged.'
        raise ValueError(f'Checkpoint conflict: {reference} already has another target; refusing overwrite.')
    # An explicit object:ref refspec creates a lightweight tag. No identity needed.
    # A concurrent writer is rejected by Git: no force or destructive ref update.
    git(repository, 'push', 'origin', sha + ':' + reference)
    return f'Created checkpoint {tag} at {sha}.'


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, default=Path.cwd())
    arguments = parser.parse_args()
    try:
        print(checkpoint(arguments.repository.resolve()))
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        detail = error.stderr.strip() if isinstance(error, subprocess.CalledProcessError) else str(error)
        print('Checkpoint refused: ' + detail, file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
