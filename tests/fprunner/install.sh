#!/bin/bash

set -euxo pipefail
cd "$(dirname "$0")"

pip install frida-tools
cd download

wget https://archive.org/download/adobeflash_debug_downloads/flash_player_sa_linux_debug.x86_64.tar.gz
tar -zxvf flash_player_sa_linux_debug.x86_64.tar.gz flashplayerdebugger
