---
name: analyze-swf
description: Inspect the contents of SWF files (header, tags, AVM1/AVM2 bytecode, decompiled ActionScript, embedded assets) using JPEXS Free Flash Decompiler. Use whenever you need to know what is inside a .swf.
---

SWFs are compressed binary files, so don't try to read them directly or guess
their contents. Use JPEXS Free Flash Decompiler (FFDec), which has a
command-line interface.

## Running FFDec

Always run FFDec through the wrapper script in this skill (paths are relative
to the repository root):

```bash
.agents/skills/analyze-swf/ffdec.sh --help
.agents/skills/analyze-swf/ffdec.sh --help <command>   # e.g. --help export
```

It uses whichever installation is available: `$FFDEC_JAR` (run with
`java -jar`), the Flathub flatpak, or `ffdec`/`ffdec.sh` on `PATH`. If it
exits with "FFDec ... not found", tell the user and suggest installing it as
the message describes. Don't search the disk for it.

This skill was written against FFDec 26.3.0. Its command-line interface and
output formats can change between versions. If a command or flag doesn't work
as described here, check the version (first line of `--help`) and the built-in
help.

- Each invocation takes about 2 seconds to start the JVM, so batch work into as
  few calls as you can.
- Pass `-config key=value` only together with a command. Passed on its own, it
  **saves** the setting to the user's FFDec configuration.
- Export into the repo, e.g. `target/swf-analysis/`. It's gitignored, so the
  output can stay there. That location also avoids the flatpak `/tmp` problem
  below.

### Flatpak-specific issues

These apply when the script runs the flatpak.

- **Flatpak doesn't work inside a restricted command sandbox.** If your
  commands run in one, flatpak fails with
  `error: open(O_TMPFILE): Read-only file system`, and you need to run the
  command outside the sandbox or ask the user to allow it.
- **The flatpak can only see `$HOME`, and its `/tmp` is private.** That means:
  - Input files under `/tmp` (including temporary work directories) can't be
    found. Pipe them in through stdin instead:
    `-dumpSWF /dev/stdin < /tmp/.../file.swf`.
  - Output written under `/tmp` is **silently lost**: FFDec prints `OK`, but
    nothing appears on the host.

## Quick overview

```bash
.agents/skills/analyze-swf/ffdec.sh -header file.swf     # version, compression, stage size, frame rate/count
.agents/skills/analyze-swf/ffdec.sh -dumpSWF file.swf    # every tag, nested sprite tags indented
.agents/skills/analyze-swf/ffdec.sh -dumpAS3 file.swf    # AS3 classes: "<fully.qualified.Name> <script index>"
.agents/skills/analyze-swf/ffdec.sh -dumpAS2 file.swf    # AVM1 script locations, e.g. "/frame 1 - DoAction"
```

Each `-dumpSWF` line looks like this:

```
00000046:    5. FrameLabel (name: "MyLabel") tagId= 43 len=       8  4d 79 4c 61 62 65 6c 00
```

- The offset is **in the decompressed file** and points at the tag's record
  header. To look at the raw bytes, run
  `.agents/skills/analyze-swf/ffdec.sh -decompress in.swf target/swf-analysis/out.swf` and then
  `xxd -s 0x46 -l 32 target/swf-analysis/out.swf`.
- Character IDs are shown as `chid`, depths as `dpt`.
- The hex preview is cut short (`...`) for long tags.

On large files the dump is long. Summarize it before reading:

```bash
.agents/skills/analyze-swf/ffdec.sh -dumpSWF file.swf | awk '{print $3}' | sort | uniq -c | sort -rn
```

## Code

Export scripts with `-export script <outdir> <swf>`. The output goes to
`<outdir>/scripts/...`. For AS3, classes are laid out by their package path.

`-selectid` never narrows script exports, and `-selectclass` is AS3-only, so
for AVM1 every script gets exported. That's usually fine, because AVM1
decompilation is fast: about 5000 scripts take around 20 seconds. Export
everything once, then grep the output.

### AS3: narrowing the export

**Only export what you need.** A full decompile of a large SWF can be very slow.
One with several thousand classes can take around 10 minutes and produce tens of
MB of source, while a single class or package takes seconds. Find names with
`-dumpAS3` first, then narrow the export:

```bash
.agents/skills/analyze-swf/ffdec.sh -selectclass com.example.MyClass -export script target/swf-analysis/as file.swf
.agents/skills/analyze-swf/ffdec.sh -selectclass 'com.example.+' -export script ...  # one package
.agents/skills/analyze-swf/ffdec.sh -selectclass 'com.++' -export script ...        # package plus subpackages
```

If you do need a full AS3 decompile, for example so you can grep across all the
code, expect it to take minutes and make sure your command timeout allows for
that. Run it in the background if you can.

### Source vs P-code

Decompiled ActionScript is a **reconstruction** and loses detail that matters
to Ruffle. When the exact semantics matter (VM behavior, opcodes, coercions,
exact multinames, exception ranges), export P-code:

```bash
.agents/skills/analyze-swf/ffdec.sh -format script:pcode    -export script <outdir> file.swf  # disassembly
.agents/skills/analyze-swf/ffdec.sh -format script:pcodehex -export script <outdir> file.swf  # plus bytes; "; xx xx" line precedes its instruction
```

Things the decompiled source hides:

- **AVM1:** `+` can be `Add` (legacy SWF4 numeric add) or `Add2` (typed add),
  and they behave differently. Constant pool usage is hidden too. P-code also
  resolves constant pool entries to string literals by default. Add
  `-config resolveConstants=false` to see the actual `Push constantN`.
- **AVM2:** P-code shows the full multinames with namespace sets, `coerce` and
  `convert_*` ops, `maxstack`/`localcount`/scope depths, and `try from ... to ...
  target ...` exception blocks. The decompiled source renames locals and
  parameters (`_loc3_`, `param1`) whenever debug info is missing.
- **Control flow the decompiler couldn't structure** shows up as `§§goto(addrXXXX)`,
  `§§push`, `§§pop` and similar. This is common in hand-optimized or
  machine-generated code. Read the P-code for these methods.
- **CrossBridge/Alchemy code** (C/C++ compiled to AVM2, often thousands of
  classes in one package) decompiles to calls like `li32(addr)` and
  `si8(v, addr)` from `avm2.intrinsics.memory`. These are the domain-memory
  opcodes, not real function calls.
- **Identifiers that aren't valid ActionScript names** are written as `§name§`,
  e.g. `§\x01§`. They don't mean the decompilation failed.
- **Obfuscated code** (e.g. lots of `§§constant(N)` or `{invalid_utf8=...}`):
  add `-config autoDeobfuscate=true` to the export to get readable source.
  That source is even further from what Ruffle executes, so if the obfuscation
  itself might be what triggers a bug, read the original P-code.

## Assets and other tags

```bash
.agents/skills/analyze-swf/ffdec.sh -export symbolClass <outdir> file.swf   # symbols.csv, lines like: 12;"com.example.MyClass"
.agents/skills/analyze-swf/ffdec.sh -selectid 12,34 -export binaryData <outdir> file.swf  # <chid>_<ClassName>.bin
.agents/skills/analyze-swf/ffdec.sh -export image,sound,font,shape,text <outdir> file.swf
.agents/skills/analyze-swf/ffdec.sh -select 1-5 -export frame <outdir> file.swf               # render frames to PNG
```

- `-selectid` picks characters by ID (`1-5`, `2,3`, `9-`). You can combine it
  with any character export type (not `script`).
- `-export all <outdir>` exports every resource type. Run `.agents/skills/analyze-swf/ffdec.sh --help export`
  to list all types and `-format` options (e.g. `shape:svg`, `sound:wav`,
  `image:png`).
- Use `file` on the exported `.bin` files to identify them. `DefineBinaryData`
  often holds embedded files such as WAVs, SVGs, XML or nested SWFs.

### Full tag structure as XML

```bash
.agents/skills/analyze-swf/ffdec.sh -swf2xml file.swf target/swf-analysis/file.xml
```

This gives every field of every tag (flags, matrices, colors, filters, button
records, etc.), which is useful for checking parsed values against Ruffle's
`swf` crate. Bytecode (AVM1 `actionBytes`, AVM2 `codeBytes`) and binary data
stay as hex blobs, so use `-export script` for code. On big SWFs the XML is
huge; grep it rather than reading it whole.

## Caveats

- JPEXS is a third-party tool with its own parser and decompiler. If its output
  disagrees with Ruffle's parser, look at the raw bytes; don't assume either
  one is right. The SWF spec can be wrong or incomplete too. What matters is
  how Flash Player reads the SWF, so when in doubt, check its behavior.
- JPEXS may normalize or fix up malformed data it reads. When you're
  investigating a malformed SWF, confirm what's there with `-dumpSWF` hex or
  `xxd` on the decompressed file.
- The `test.swf` files in `tests/` are generated from sources. Read the
  test's source first (`Test.as`, `.fla` notes, `test.toml`), and use JPEXS when
  there is no source or you need to see exactly what the compiler emitted.
