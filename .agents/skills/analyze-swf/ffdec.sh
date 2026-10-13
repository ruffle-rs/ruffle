#!/usr/bin/env bash
# Runs JPEXS Free Flash Decompiler (FFDec) with the given arguments, using
# whichever installation is available.

set -euo pipefail

# Without a command-line option (e.g. no arguments, or just a file), FFDec
# opens its GUI.
if [[ $# -eq 0 || "$1" != -* ]]; then
    echo "Usage: $0 <-command> [args...]   (see --help)" >&2
    exit 2
fi

if [[ -n "${FFDEC_JAR:-}" ]]; then
    exec java -jar "$FFDEC_JAR" "$@"
fi

if command -v flatpak >/dev/null && flatpak info com.jpexs.decompiler.flash >/dev/null 2>&1; then
    # Drop flatpak's harmless "F: Not sharing ... with sandbox" warnings.
    exec flatpak run com.jpexs.decompiler.flash "$@" 2> >(grep -v '^F: ' >&2)
fi

for cmd in ffdec ffdec.sh; do
    if command -v "$cmd" >/dev/null; then
        exec "$cmd" "$@"
    fi
done

echo "FFDec (JPEXS Free Flash Decompiler) not found.
Install it with:
    flatpak install flathub com.jpexs.decompiler.flash
or download it from https://github.com/jpexs/jpexs-decompiler/releases
and set FFDEC_JAR=/path/to/ffdec.jar." >&2
exit 127
