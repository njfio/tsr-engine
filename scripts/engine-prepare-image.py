#!/usr/bin/env python3
"""Prepare locked crates.io sources in a new local image; never execute downloaded code.
Input is an initialized engine admission. Output is a new spec for a NEW engine run.
"""
import argparse
import hashlib
import gzip
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import time
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'
ARCHIVE_LIMIT = 10 * 1024 * 1024
TOTAL_LIMIT = 64 * 1024 * 1024
EXPANDED_LIMIT = 128 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        raise ValueError('download redirects refused')


def download(url):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    deadline = time.monotonic() + 45
    with opener.open(urllib.request.Request(url, headers={'User-Agent': 'tsr-locked-preparation/1'}), timeout=10) as response:
        require(response.status == 200, 'download did not return 200')
        chunks, length = [], 0
        while True:
            require(time.monotonic() < deadline, 'download deadline exceeded')
            chunk = response.read(65536)
            if not chunk:
                break
            length += len(chunk)
            require(length <= ARCHIVE_LIMIT, 'crate archive exceeds 10 MiB')
            chunks.append(chunk)
        return b''.join(chunks)


def packages(spec):
    require(spec.get('cargo') is True, 'Cargo admission required')
    require(re.fullmatch(r'sha256:[a-f0-9]{64}', spec['image']), 'pinned image required')
    lock = tomllib.loads(spec['protected']['Cargo.lock'])
    require(len(lock['package']) <= 32, 'lock inventory exceeds 32 packages')
    result, seen = [], set()
    for entry in lock['package']:
        if 'source' not in entry:
            continue
        require(entry['source'] == REGISTRY, 'only crates.io sources supported')
        name, version, checksum = entry['name'], entry['version'], entry['checksum']
        require(re.fullmatch(r'[A-Za-z][A-Za-z0-9_-]{0,63}', name), 'invalid crate name')
        require(re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version), 'stable numeric version required')
        require(re.fullmatch(r'[a-f0-9]{64}', checksum), 'SHA256 checksum required')
        identity = f'{name}-{version}'
        require(identity.lower() not in seen, 'duplicate crate identity')
        seen.add(identity.lower())
        result.append((identity, checksum, f'https://static.crates.io/crates/{name}/{identity}.crate'))
    require(result, 'no registry dependencies in admission')
    return result


def unpack(data, identity, checksum, vendor, remaining):
    require(sha(data) == checksum, 'archive checksum mismatch')
    destination = vendor / identity
    destination.mkdir()
    files, seen, expanded = {}, set(), 0
    with gzip.GzipFile(fileobj=io.BytesIO(data)) as compressed:
        tar_data = compressed.read(EXPANDED_LIMIT + 1)
    require(len(tar_data) <= EXPANDED_LIMIT, 'decompressed archive exceeds 128 MiB')
    with tarfile.open(fileobj=io.BytesIO(tar_data), mode='r:') as archive:
        for count, member in enumerate(archive, 1):
            require(count <= 10000, 'crate exceeds 10000 archive entries')
            path = PurePosixPath(member.name)
            require(not path.is_absolute() and '..' not in path.parts and '\\' not in member.name, 'unsafe archive path')
            require(path.parts and path.parts[0] == identity, 'unexpected archive root')
            require(member.isdir() or member.isfile(), 'archive links and special files refused')
            relative = PurePosixPath(*path.parts[1:])
            if str(relative) == '.':
                require(member.isdir(), 'archive root must be a directory')
                continue
            require(len(str(relative)) <= 512, 'archive path too long')
            folded = str(relative).lower()
            require(folded not in seen, 'duplicate/case-colliding archive path')
            seen.add(folded)
            require(str(relative) != '.cargo-checksum.json', 'archive supplied checksum metadata')
            target = destination.joinpath(*relative.parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            expanded += member.size
            require(member.size >= 0 and expanded <= remaining, 'expanded dependency bytes exceed limit')
            target.parent.mkdir(parents=True, exist_ok=True)
            contents = archive.extractfile(member).read(member.size + 1)
            require(len(contents) == member.size, 'incomplete archive member')
            with target.open('xb') as output:
                output.write(contents)
            target.chmod(0o755 if member.mode & 0o111 else 0o644)
            files[str(relative)] = sha(contents)
    require('Cargo.toml' in files, 'crate manifest missing')
    (destination / '.cargo-checksum.json').write_text(json.dumps({'package': checksum, 'files': files}, sort_keys=True))
    return expanded


def run(args):
    # Read the controller's saved immutable admission, not candidate files or repository configuration.
    command = [str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(args.directory), 'project-config']
    spec = json.loads(subprocess.check_output(command, timeout=10))
    inventory = packages(spec)
    out = args.out.resolve()
    require(not out.is_relative_to(args.directory.resolve()), 'output must be outside the existing engine directory')
    out.mkdir(parents=True, exist_ok=False)
    context = out / 'context'
    vendor = context / 'vendor'
    vendor.mkdir(parents=True)
    records, downloaded, expanded = [], 0, 0
    for identity, checksum, url in inventory:
        data = download(url)
        downloaded += len(data)
        require(downloaded <= TOTAL_LIMIT, 'downloads exceed 64 MiB')
        expanded += unpack(data, identity, checksum, vendor, EXPANDED_LIMIT - expanded)
        records.append({'crate': identity, 'url': url, 'sha256': checksum, 'bytes': len(data)})
    cargo = context / 'cargo'
    cargo.mkdir()
    (cargo / 'config.toml').write_text('[source.crates-io]\nreplace-with = "tsr-vendor"\n[source.tsr-vendor]\ndirectory = "/opt/tsr-vendor"\n[net]\noffline = true\n')
    # No RUN instruction: neither application nor dependency code executes during preparation.
    (context / 'Dockerfile').write_text(f'FROM {spec["image"]}\nCOPY vendor/ /opt/tsr-vendor/\nCOPY cargo/ /opt/tsr-cargo/\nENV CARGO_HOME=/opt/tsr-cargo\n')
    endpoint = subprocess.check_output(['docker', 'context', 'inspect', '--format', '{{.Endpoints.docker.Host}}'], text=True, timeout=10).strip()
    require(endpoint.startswith('unix:///'), 'only the local Docker Unix socket is supported')
    config = out / 'docker-config'
    config.mkdir()
    env = {'PATH': os.environ.get('PATH', '/usr/bin:/bin'), 'HOME': str(config),
           'DOCKER_CONFIG': str(config), 'DOCKER_HOST': endpoint, 'DOCKER_BUILDKIT': '0'}
    # A local-only build with empty client configuration; no credential helpers or build secrets.
    subprocess.run(['docker', 'image', 'inspect', spec['image']], env=env, check=True, stdout=subprocess.DEVNULL, timeout=15)
    with (out / 'build.log').open('w') as log:
        subprocess.run(['docker', 'build', '--pull=false', '--network=none', '--iidfile', str(out / 'image.id'), str(context)],
                       env=env, check=True, stdout=log, stderr=subprocess.STDOUT, timeout=120)
    image = (out / 'image.id').read_text().strip()
    require(re.fullmatch(r'sha256:[a-f0-9]{64}', image), 'build did not return an immutable image id')
    original = json.dumps(spec, sort_keys=True).encode()
    base = spec['image']
    spec['image'] = image
    (out / 'prepared-spec.json').write_text(json.dumps(spec, indent=2) + '\n')
    result = {'base_image': base, 'image': image, 'original_admission_sha256': sha(original),
              'lock_sha256': sha(spec['protected']['Cargo.lock'].encode()), 'packages': records,
              'downloaded_bytes': downloaded, 'expanded_bytes': expanded,
              'prepared_spec': str(out / 'prepared-spec.json'), 'application_code_executed': False,
              'dependency_code_executed': False, 'build_network': 'none'}
    (out / 'preparation.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True, help='New directory, never overwritten')
    try:
        print(json.dumps(run(parser.parse_args()), indent=2))
    except Exception as error:
        raise SystemExit(f'Preparation stopped: {error}')
