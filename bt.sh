#!/usr/bin/env bash

# sauce : https://stackoverflow.com/a/63901492
# kredit: https://blog.cryptomilk.org/2010/12/23/gdb-backtrace-to-file/

ex=(
    -ex "run"
    -ex "set logging overwrite off"
    -ex "set logging file gdb.bt"
    -ex "set logging on"
    -ex "set pagination off"
    -ex "handle SIG33 pass nostop noprint"
    -ex "echo backtrace:\n"
    -ex "backtrace full"
    -ex "echo \n\nregisters:\n"
    -ex "info registers"
    -ex "echo \n\ncurrent instructions:\n"
    -ex "x/16i \$pc"
    -ex "echo \n\nthreads backtrace:\n"
    -ex "thread apply all backtrace"
    -ex "set logging off"
    -ex "quit"
)
echo 0 | gdb -batch-silent "${ex[@]}" --args "$@"

