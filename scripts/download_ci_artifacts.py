#!/usr/bin/env python3
"""Download exact selected artifact IDs, verify their digests, and extract safely."""
from __future__ import annotations

import argparse
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import zipfile
import zlib


TRACE_NAME = 'ARTIFACT_SELECTION.json'


def validate_selection(selection: object) -> dict:
    """Validate the download boundary independently of the artifact selector."""
    if not isinstance(selection, dict) or type(selection.get('schema')) is not int or selection['schema'] != 1:
        raise ValueError('selection must have schema 1')
    repository = selection.get('repository')
    if (not isinstance(repository, str)
            or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9-]*/[A-Za-z0-9_.-]+', repository)
            or repository.split('/')[1] in ('.', '..')):
        raise ValueError('selection repository must be owner/repo')
    run_id = selection.get('run_id')
    if type(run_id) is not int or run_id <= 0:
        raise ValueError('selection run_id must be a positive integer')
    artifacts = selection.get('artifacts')
    if not isinstance(artifacts, list) or not artifacts:
        raise ValueError('selection must contain artifacts')
    identifiers, names = set(), set()
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            raise ValueError('artifact metadata must be an object')
        identifier, name = artifact.get('id'), artifact.get('name')
        if type(identifier) is not int or identifier <= 0 or identifier in identifiers:
            raise ValueError('artifact IDs must be unique positive integers')
        if (not isinstance(name, str) or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*', name)
                or name.endswith('.') or name.casefold() == TRACE_NAME.casefold() or name in names):
            raise ValueError('artifact names must be unique safe directory names')
        digest = artifact.get('digest')
        if not isinstance(digest, str) or not re.fullmatch(r'sha256:[0-9a-fA-F]{64}', digest):
            raise ValueError(f'artifact {identifier} requires a SHA256 digest')
        if artifact.get('expired') is not False:
            raise ValueError(f'artifact {identifier} is expired or lacks expiry metadata')
        created_at = artifact.get('created_at')
        try:
            created = datetime.fromisoformat(created_at.replace('Z', '+00:00'))
        except (AttributeError, TypeError, ValueError):
            raise ValueError(f'artifact {identifier} has an invalid creation timestamp') from None
        if created.utcoffset() is None:
            raise ValueError(f'artifact {identifier} requires a timezone-aware creation timestamp')
        workflow_run = artifact.get('workflow_run')
        if (not isinstance(workflow_run, dict) or type(workflow_run.get('id')) is not int
                or workflow_run['id'] != run_id):
            raise ValueError(f'artifact {identifier} does not belong to the selected workflow run')
        identifiers.add(identifier)
        names.add(name)
    return selection


def extract_archive(archive_file, destination: Path) -> None:
    """Reject unsafe names, links, duplicate targets, and CRC failures before writes."""
    try:
        with zipfile.ZipFile(archive_file) as archive:
            members = {}
            for member in archive.infolist():
                name = member.orig_filename
                parts = name.removesuffix('/').split('/')
                if (not name or name.startswith('/') or '\\' in name or ':' in name
                        or PureWindowsPath(name).drive or any(ord(char) < 32 or ord(char) == 127 for char in name)
                        or any(part in ('', '.', '..') for part in parts)):
                    raise ValueError('artifact ZIP contains an unsafe member path')
                path = PurePosixPath(*parts)
                if path in members:
                    raise ValueError('artifact ZIP contains duplicate member paths')
                kind = stat.S_IFMT(member.external_attr >> 16)
                if kind not in (0, stat.S_IFREG, stat.S_IFDIR):
                    raise ValueError('artifact ZIP contains a symlink or special file')
                if (kind == stat.S_IFDIR and not member.is_dir()) or (kind == stat.S_IFREG and member.is_dir()):
                    raise ValueError('artifact ZIP has conflicting file type metadata')
                members[path] = member
            files = {path for path, member in members.items() if not member.is_dir()}
            if any(parent in files for path in members for parent in path.parents):
                raise ValueError('artifact ZIP has a file/directory path conflict')
            if archive.testzip() is not None:
                raise ValueError('artifact ZIP failed CRC validation')
            destination.mkdir()
            for path, member in members.items():
                target = destination.joinpath(*path.parts)
                if member.is_dir():
                    target.mkdir(parents=True, exist_ok=True)
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    with archive.open(member) as source, target.open('xb') as output:
                        shutil.copyfileobj(source, output)
    except (zipfile.BadZipFile, RuntimeError, NotImplementedError, zlib.error, EOFError):
        raise ValueError('artifact ZIP is invalid or failed integrity validation') from None


def download_artifacts(selection_path: Path, output: Path) -> int:
    raw = selection_path.read_bytes()
    selection = validate_selection(json.loads(raw))
    if not os.environ.get('GH_TOKEN'):
        raise ValueError('GH_TOKEN is required to download selected artifacts')
    try:
        output.mkdir(parents=True)
    except FileExistsError:
        raise ValueError('output already exists; never overwrite downloaded evidence') from None
    try:
        for artifact in selection['artifacts']:
            identifier = artifact['id']
            endpoint = f"repos/{selection['repository']}/actions/artifacts/{identifier}/zip"
            with tempfile.TemporaryFile() as archive_file:
                try:
                    # gh follows the API's signed redirect without forwarding the API token.
                    # Inherit GH_TOKEN; never put it in argv or expose gh's response/error body.
                    subprocess.run(['gh', 'api', '--hostname', 'github.com', endpoint],
                                   stdout=archive_file, stderr=subprocess.PIPE, check=True)
                except subprocess.CalledProcessError as error:
                    raise ValueError(f'artifact {identifier} download failed (gh exit {error.returncode})') from None
                except OSError:
                    raise ValueError('GitHub CLI (gh) could not be started') from None
                archive_file.seek(0)
                digest = hashlib.sha256()
                for chunk in iter(lambda: archive_file.read(1024 * 1024), b''):
                    digest.update(chunk)
                if digest.hexdigest() != artifact['digest'].split(':', 1)[1].lower():
                    raise ValueError(f'artifact {identifier} SHA256 digest mismatch')
                archive_file.seek(0)
                extract_archive(archive_file, output / artifact['name'])
        (output / TRACE_NAME).write_bytes(raw)
    except Exception:
        # This directory was created exclusively above; never leave partial evidence.
        shutil.rmtree(output)
        raise
    return len(selection['artifacts'])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--selection', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        count = download_artifacts(args.selection, args.output)
    except (OSError, ValueError) as error:
        print(f'Artifact download failed: {error}', file=sys.stderr)
        return 1
    print(f'Downloaded and verified {count} selected artifacts.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
