#!/usr/bin/env python3

import os
import re
import subprocess
import sys
from collections import namedtuple

Issue = namedtuple('Issue', ['level', 'message'])

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
REPO_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, '../../'))

# ===== Utilities ==========================================

def log(msg):
    print(msg, file=sys.stderr)

def run_command(args, cwd=REPO_DIR):
    return subprocess.run(
        args,
        cwd=cwd,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout.decode('utf-8')

def get_commits(base_sha, head_sha):
    log_output = run_command([
        'git', 'log',
        f'{base_sha}..{head_sha}',
        '--format=%H%x1f%P%x1f%an <%ae>%x1f%s%x1f%b%x1f%B%x1e',
    ])

    commits = []
    for record in log_output.split('\x1e'):
        record = record.strip('\n')
        if not record:
            continue
        sha, parents, author, subject, body, raw = record.lstrip('\n').split('\x1f', 5)
        commits.append({
            'sha': sha,
            'parents': parents.split(),
            'author': author,
            'subject': subject,
            'body': body,
            'raw': raw,
        })
    return commits

# ===== Commit Review ===========================================

CO_AUTHORED_BY_RE = re.compile(r'^Co-authored-by:\s*(.*)\s*$', re.IGNORECASE | re.MULTILINE)

FIXUP_RE = re.compile(r'^(fixup|squash)!|^wip\b', re.IGNORECASE)

# Emails known to be appended as the author or a Co-authored-by trailer
# by popular AI coding agents. Not exhaustive -- add more as they come up.
AI_AGENT_EMAILS = {
    'agent@cursor.com',  # Cursor
    'agent@replit.com',  # Replit Agent
    'agent@warp.dev',  # Warp
    'amazon-q[bot]@users.noreply.github.com',  # Amazon Q Developer
    'amp@ampcode.com',  # Amp
    'codex@openai.com',  # OpenAI Codex
    'cody@sourcegraph.com',  # Sourcegraph Cody
    'copilot@github.com',  # Copilot
    'crush@charm.land',  # Crush
    'devin-ai-integration[bot]@users.noreply.github.com',  # Devin (GitHub App)
    'hello@sketch.dev',  # Sketch
    'junie@jetbrains.com',  # Junie
    'noreply@aider.chat',  # Aider
    'noreply@anthropic.com',  # Claude Code
    'noreply@codeium.com',  # Codeium / Windsurf
    'noreply@opencode.ai',  # OpenCode
    'openhands@all-hands.dev',  # OpenHands
}

def contains_ai_agent_email(s):
    s = s.lower()
    for agent_email in AI_AGENT_EMAILS:
        if agent_email in s:
            return True

    return False

def contains_noreply_email(s):
    s = s.lower()

    # Allow GitHub's noreply emails as an exception.
    if 'noreply' in s and '@users.noreply.github.com' not in s:
        return True

    return False

def review_commit(commit):
    issues = []

    subject = commit['subject']
    author = commit['author']
    body = commit['body']
    raw_lines = commit['raw'].splitlines()

    # Check subject is not empty.
    if not subject.strip():
        issues.append(Issue('error', 'Subject line cannot be empty.'))

    # Check subject length.
    if len(subject) > 72:
        issues.append(Issue(
            'warning',
            f"Subject line with {len(subject)} characters is too long, "
            "it's best to keep it 50 characters or less.",
        ))

    # Check body line lengths.
    if any(len(line) > 72 for line in body.splitlines()):
        issues.append(Issue(
            'warning',
            'Body has line(s) longer than 72 characters. '
            'Text should wrap at 72 characters.',
        ))

    # Warn on commits with no body.
    if not body.strip():
        issues.append(Issue(
            'warning',
            'Commit has no body -- consider describing the change.',
        ))

    # Check for a blank line separating subject from body.
    if len(raw_lines) > 1 and raw_lines[1].strip():
        issues.append(Issue(
            'error',
            'Missing a blank line between the subject and the body.',
        ))

    # Disallow leftover fixup/squash/WIP commits.
    if FIXUP_RE.match(subject):
        issues.append(Issue(
            'error',
            'Looks like a fixup/squash/WIP commit -- squash it before merging.',
        ))

    # Disallow merge commits.
    if len(commit['parents']) > 1:
        issues.append(Issue(
            'error',
            'A merge commit found -- rebase instead of merging.',
        ))

    # Disallow noreply author emails.
    if contains_noreply_email(author):
        issues.append(Issue(
            'warning',
            f'Author uses a "noreply" email ({author}).',
        ))
    for coauthor in CO_AUTHORED_BY_RE.findall(body):
        if contains_noreply_email(coauthor):
            issues.append(Issue(
                'warning',
                f'Co-authored-by tag uses a "noreply" email ({coauthor}).',
            ))

    # Disallow known AI agent emails as authors.
    if contains_ai_agent_email(author):
        issues.append(Issue(
            'error',
            f'Author email ({author}) belongs to a known AI coding agent. '
            'We do not accept fully autonomous agent submissions, '
            'we require a human in the loop.',
        ))
    for coauthor in CO_AUTHORED_BY_RE.findall(body):
        if contains_ai_agent_email(coauthor):
            issues.append(Issue(
                'error',
                f'Co-authored-by tag ({coauthor}) belongs to a known AI coding '
                'agent. Agents cannot claim authorship and cannot be contacted.',
            ))

    return issues

# ===== Commands ===========================================

def commits(base_sha, head_sha):
    """
    Review the commits added by a PR (base_sha..head_sha).
    """

    had_error = False

    for commit in get_commits(base_sha, head_sha):
        issues = review_commit(commit)
        for issue in issues:
            sha = commit['sha'][:8]
            print(f'::{issue.level} title=Commit review::{sha}: {issue.message}')
            if issue.level == 'error':
                had_error = True

    if had_error:
        sys.exit(1)

def main():
    cmd = sys.argv[1]
    log(f'Running command {cmd}')
    if cmd == 'commits':
        base_sha = sys.argv[2]
        head_sha = sys.argv[3]
        commits(base_sha, head_sha)


if __name__ == '__main__':
    main()
