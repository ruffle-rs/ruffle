#!/usr/bin/env python3

import os
import subprocess
import sys
from datetime import datetime
import xml.etree.ElementTree as xml
import json

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
REPO_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, '../../'))


# ===== Utilities ==========================================

def get_current_date():
    now = datetime.now()
    return now.strftime('%Y-%m-%d')


def get_current_time_version():
    now = datetime.now()
    return f'{now.year}.{now.month}.{now.day}'


def get_current_day_id():
    now = datetime.now()
    day = now.strftime('%j')
    return f'{now.year - 2000}{day}'


def get_tag_name():
    now = datetime.now()
    current_time_dashes = now.strftime('%Y-%m-%d')
    tag_name = f'nightly-{current_time_dashes}'
    return tag_name


def github_output(variable, value):
    line = f'{variable}={value}'
    print(line)
    if 'GITHUB_OUTPUT' in os.environ:
        with open(os.environ['GITHUB_OUTPUT'], 'a') as f:
            f.write(line + '\n')


def log(msg):
    print(msg, file=sys.stderr)


def add_release_to_metainfo(path, tag_name, version):
    url = f'https://github.com/ruffle-rs/ruffle/releases/tag/{tag_name}'
    xml_release = xml.Element('release')
    xml_release.set('version', version)
    xml_release.set('date', get_current_date())
    xml_url = xml.Element('url')
    xml_url.text = url
    xml_release.append(xml_url)
    xml_doc = xml.parse(path)
    xml_releases = xml_doc.getroot().find('releases')
    xml_releases.insert(0, xml_release)
    xml.indent(xml_doc, space="    ")

    xml_bytes = xml.tostring(xml_doc.getroot(), encoding='utf-8', xml_declaration=False)

    # We don't want a space before closing XML brackets.
    # It's the easiest way to do this, and unlikely to break the XML,
    # as we don't have any such content within metainfo.
    xml_bytes = xml_bytes.replace(b' />\n', b'/>\n')

    with open(path, 'wb') as fd:
        fd.write(b'<?xml version="1.0" encoding="utf-8"?>\n')
        fd.write(xml_bytes)
        fd.write(b'\n')


# ===== Commands to execute ================================

def run_command(args, cwd=REPO_DIR):
    return subprocess.run(
        args,
        cwd=cwd,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout.decode('utf-8')


def cargo_get_version():
    return run_command(['cargo', 'get', 'workspace.package.version']).strip()


def cargo_set_version(args):
    run_command(['cargo', 'set-version', '--exclude', 'swf', *args])


def gh_list_nightly_tags(limit):
    tags_json = run_command([
        'gh', 'release', 'list',
        '--order', 'desc',
        '--limit', str(limit),
        '--json=tagName',
        '--jq', '.[] | select(.tagName | startswith("nightly-"))',
    ])
    for tag_json in tags_json.splitlines():
        tag = json.loads(tag_json)
        yield tag['tagName']


def gh_get_last_nightly_tag():
    # Assume that the last nightly tag will be in the last 16 releases.
    # It's very unlikely we'll have 16 releases without a nightly.
    return next(gh_list_nightly_tags(16), None)


# ===== Commands ===========================================

def bump():
    """
    Bump the current version of Ruffle nightly.
    """

    current_version = cargo_get_version()
    log(f'Current version: {current_version}')

    log('Bumping minor version to get the next planned version')
    cargo_set_version(['--bump', 'minor'])
    next_planned_version = cargo_get_version()
    run_command(['git', 'reset', '--hard', 'HEAD'])

    log(f'Next planned version is {next_planned_version}')

    nightly_version = f'{next_planned_version}-nightly.{get_current_time_version()}'
    log(f'Nightly version is {nightly_version}')

    cargo_set_version([nightly_version])

    version = cargo_get_version()
    version4 = f'{next_planned_version}.{get_current_day_id()}'

    npm_dir = f'{REPO_DIR}/web'
    run_command(['npm', 'install', 'workspace-version'], cwd=npm_dir)
    run_command(['npm', 'version', '--no-git-tag-version', version], cwd=npm_dir)
    run_command(['./node_modules/workspace-version/dist/index.js'], cwd=npm_dir)

    github_output('current-version', current_version)
    github_output('version', version)
    github_output('version4', version4)


def metainfo():
    metainfo_path1 = f'{REPO_DIR}/desktop/packages/linux/rs.ruffle.Ruffle.metainfo.xml'
    metainfo_path2 = f'{REPO_DIR}/desktop/packages/linux/rs.ruffle.Ruffle.metainfo.xml.in'
    version = cargo_get_version()
    tag_name = get_tag_name()
    add_release_to_metainfo(metainfo_path1, tag_name, version)
    add_release_to_metainfo(metainfo_path2, tag_name, version)


def commit():
    commit_message = f'Release {cargo_get_version()}'
    run_command(['git', 'config', 'user.name', 'RuffleBuild'])
    run_command(['git', 'config', 'user.email', 'ruffle@ruffle.rs'])
    run_command(['git', 'add', '--update'])
    run_command(['git', 'commit', '-m', commit_message])


def tag_and_push():
    tag_name = get_tag_name()
    run_command(['git', 'tag', tag_name])
    run_command(['git', 'push', 'origin', 'tag', tag_name])
    github_output('tag_name', tag_name)


def release():
    """
    Create a release of Ruffle on GitHub.
    """

    now = datetime.now()
    current_time_dashes = now.strftime('%Y-%m-%d')
    current_time_underscores = now.strftime('%Y_%m_%d')

    tag_name = get_tag_name()
    last_nightly_tag = gh_get_last_nightly_tag()
    release_name = f'Nightly {current_time_dashes}'
    package_prefix = f'ruffle-nightly-{current_time_underscores}'
    release_options = ['--generate-notes', '--prerelease']

    if last_nightly_tag is not None:
        log(f'Using {last_nightly_tag} as start tag for notes')
        release_options += ['--notes-start-tag', last_nightly_tag]
    else:
        log('No start tag for notes found')

    run_command([
        'gh', 'release', 'create', tag_name,
        '--title', release_name,
        '--verify-tag',
        *release_options])

    github_output('tag_name', tag_name)
    github_output('package_prefix', package_prefix)


def main():
    cmd = sys.argv[1]
    log(f'Running command {cmd}')
    if cmd == 'bump':
        bump()
    elif cmd == 'metainfo':
        metainfo()
    elif cmd == 'commit':
        commit()
    elif cmd == 'tag-and-push':
        tag_and_push()
    elif cmd == 'release':
        release()


if __name__ == '__main__':
    main()
