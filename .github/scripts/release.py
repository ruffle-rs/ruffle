#!/usr/bin/env python3
import os
import subprocess
import sys
from datetime import datetime
import xml.etree.ElementTree as xml

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
REPO_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, '../../'))


# ===== Utilities ==========================================

def add_release_to_metainfo(path):
    version = cargo_get_version()
    url = f'https://github.com/ruffle-rs/ruffle/releases/tag/v{version}'

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
    xml_doc.write(path, encoding='utf-8', xml_declaration=True)
    with open(path, 'a') as fd:
        fd.write('\n')


def get_current_date():
    now = datetime.now()
    return now.strftime('%Y-%m-%d')


def get_current_time_version():
    now = datetime.now()
    return f'{now.year}.{now.month}.{now.day}'


def get_current_day_id():
    now = datetime.now()
    day = now.strftime('%j')
    return f'{now.year - 2020}{day}'


def github_output(variable, value):
    line = f'{variable}={value}'
    print(line)
    if 'GITHUB_OUTPUT' in os.environ:
        with open(os.environ['GITHUB_OUTPUT'], 'a') as f:
            f.write(line + '\n')


def log(msg):
    print(msg, file=sys.stderr)


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


# ===== Commands ===========================================

def bump(mode):
    """
    Bump the current version of Ruffle according to the mode, which may be one of: nightly, major, minor, patch.
    """

    current_version = cargo_get_version()
    log(f'Current version: {current_version}, bumping with mode {mode}')

    if mode == 'nightly':
        cargo_set_version(['--bump', 'minor'])
        next_planned_version = cargo_get_version()
        run_command(['git', 'reset', '--hard', 'HEAD'])
        cargo_set_version([f'{next_planned_version}-nightly.{get_current_time_version()}'])
        version4 = f'{current_version}.{get_current_day_id()}'
    else:
        cargo_set_version(['--bump', mode])
        version4 = cargo_get_version()
        add_release_to_metainfo(f'{REPO_DIR}/desktop/packages/linux/rs.ruffle.Ruffle.metainfo.xml')

    version = cargo_get_version()

    npm_dir = f'{REPO_DIR}/web'
    run_command(['npm', 'install', 'workspace-version'], cwd=npm_dir)
    run_command(['npm', 'version', '--no-git-tag-version', version], cwd=npm_dir)
    run_command(['./node_modules/workspace-version/dist/index.js'], cwd=npm_dir)

    github_output('current-version', current_version)
    github_output('version', version)
    github_output('version4', version4)


def commit():
    commit_message = f'Release {cargo_get_version()}'
    run_command(['git', 'add', '--update'])
    run_command(['git', 'commit', '-m', commit_message])


def release(channel):
    """
    Create a release of Ruffle on GitHub.
    """

    if channel == 'nightly':
        now = datetime.now()
        current_time_dashes = now.strftime('%Y-%m-%d')
        current_time_underscores = now.strftime('%Y_%m_%d')

        tag_name = f'nightly-{current_time_dashes}'
        release_name = f'Nightly {current_time_dashes}'
        package_prefix = f'ruffle-nightly-{current_time_underscores}'
        release_options = ['--generate-notes', '--prerelease']
    else:
        version = cargo_get_version()
        tag_name = f'v{version}'
        release_name = f'Release {version}'
        package_prefix = f'ruffle-{version}'
        release_options = []

    release_commit = run_command(['git', 'rev-parse', 'HEAD']).strip()
    run_command([
        'gh', 'release', 'create', tag_name,
        '--title', release_name,
        '--target', release_commit,
        *release_options])

    github_output('tag_name', tag_name)
    github_output('package_prefix', package_prefix)


def main():
    cmd = sys.argv[1]
    log(f'Running command {cmd}')
    if cmd == 'bump':
        bump(sys.argv[2])
    elif cmd == 'commit':
        commit()
    elif cmd == 'release':
        release(sys.argv[2])


if __name__ == '__main__':
    main()
