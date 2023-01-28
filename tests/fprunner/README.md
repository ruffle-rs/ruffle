# fprunner


This directory contains tools for running Adobe Flash Player and capturing its output. 
It's used by our regression test suite to automatically run flashplayer on our test SWFS,
and compare the trace output to the expected output.

## Installation

Run `./install.sh` to download all of the dependencies

## Trace output capturing

flashplayer always writes its logs to the same location - on Linux, this is `~/.macromedia/Flash_Player/Logs/flashlog.txt`.
In order to run multiple instances in parallel, we need to redirect the output to a different file (per execution)
to obtain the logs for a specific SWF run.

This is accomplished using Frida (https://frida.re/). The 'agent.js' script intercepts calls to the
C stdlib function `open` - if the provided path is the flashplayer logfile, we replace it with a
new relative path. Our test suite launches each instance of flashplayer from a new temporary directory,
causing the injected relative path to resolve to that directory.

## Platform support

Currently, `install.sh` only downloads the Linux version of flashplayer, and `agent.js` will only
work on UNIX-like operating systems (since it redirects `open`). However, Frida also supports macOS and Windows
so it should be possible to extend `fprunner` to work on these platforms as well.