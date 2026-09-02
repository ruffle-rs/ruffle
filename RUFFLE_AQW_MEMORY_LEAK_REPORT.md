# Ruffle / AdventureQuest Worlds — memory leak: diagnosis, fix and validation

## 1. Executive summary

**What was wrong.** Desktop Ruffle never released a SWF once it had been
loaded. Every map, item, armour, weapon and interface asset that AQW loaded
through `Loader` stayed fully resident — its characters, its decoded bitmaps,
its SWF data and its ActionScript state — for the rest of the session, whether
or not anything could still reach it. Memory therefore grew in proportion to
the number of zone changes and equipment swaps, exactly as reported, and never
reached a steady state.

**What caused it.** Ruffle stores one `MovieLibrary` per `SwfMovie` in a map
that is *weakly keyed on the movie*, on the assumption that a library dies with
its movie. It cannot: the library holds a strong `Arc<SwfMovie>` of its own,
and so does nearly every character inside it. The map value therefore keeps its
own key alive, the key's strong count never falls to zero, and the entry is
immortal by construction. Two further problems kept the same content alive even
once that was addressed: `Avm2ClassRegistry` traced its class keys, making every
`SymbolClass` class a GC root that pinned its translation unit and hence its
movie; and the garbage collector paced itself only against the size of its own
allocations, which for a movie library is a rounding error next to the buffers
those allocations point at.

**What changed — first attempt (`463b742fe`, rejected).** Four source-level
changes in `ruffle_core`, described in section 6: a weakly-held content-root
handle that let a loaded movie's library be dropped once its content was
unreachable; weak keys in the AVM2 class registry; external-memory accounting so
the collector paces itself against the memory actually in play; and a real
implementation of `Loader.unloadAndStop`. It fixed the leak under the agreed
reproduction (sections 8 and 9), but **authenticated live AQW testing found a
correctness defect in it** (section 15): it released a movie's library while
ActionScript still held a class from that movie, which is exactly AQW's
equipment pattern, and equipment then failed with `Error #2136`.

**What changed — corrected fix (section 16).** The release condition was
replaced by a garbage-collection model. A loaded movie's library is no longer a
root that keeps everything in it alive, nor is it dropped on a single handle;
it is treated as an *ephemeron*: the collector traces only the things outside
the library that could still need it (the loaded content, the movie's
ActionScript code, and live instances of its characters), and at the end of
every marking phase a finalization pass keeps the libraries something reached
and drops the rest. That is Flash's rule — a definition lives as long as its
`ApplicationDomain` or anything else that reaches it — and it is what AQW's own
loader design (per-map domains, a replaceable shared domain, an LRU of per-slot
domains) relies on. The class registry, external-memory accounting and
`unloadAndStop` changes from the first attempt were re-evaluated and kept.

**Is it fixed.** Yes, in both directions, and both are tests. A class held out
of an unloaded SWF's domain still instantiates with all its characters (the
exact failure `463b742fe` introduced, reproduced with a real AQW asset and with
a synthetic one, both passing); and once nothing references a loaded SWF it is
released, so repeated loads no longer grow. Section 16 has the measurements,
the full test-suite results and the repeated live AQW session.

**Follow-up (section 17).** The client's first run of the corrected fix on
Windows reported lag and 3.5 GB after ten map switches. The lag was a
collection storm in the new `unloadAndStop` and is fixed; the memory was
what each of the several hundred assets AQW keeps resolvable cost in Ruffle
- eager, permanent tessellation and parsed shape records, and bitmap memory
the collector could not see - and those costs were cut by making them
on-demand and reporting them. Section 17 has the measurements, and the
memory report now says where every megabyte is so that the client's own run
can be read.


## 2. Environment

| | |
|---|---|
| OS | Ubuntu 24.04.4 LTS, Linux 7.0.0-30-generic, x86_64 |
| CPU / RAM | 4 cores, 7.7 GiB |
| GPU | Intel Haswell (Mesa, Vulkan) |
| rustc | 1.96.1 (31fca3adb 2026-06-26) |
| cargo | 1.96.1 (356927216 2026-06-26) |
| Repository | `/home/farhan/ruffle`, remote `https://github.com/ruffle-rs/ruffle` |
| Baseline commit | `89f16f4cccf4a8c58e5c5d6902edf66999440c55` (`v0.5.0-238-g89f16f4cc`) |
| Branch | `fix/aqw-memory-leak` |
| Build | `cargo build --release --package ruffle_desktop` |

No credentials, tokens or account data are reproduced anywhere in this report or
in the committed changes.

A client-supplied AQW test account does exist, and it was deliberately **not**
used. AQW's terms prohibit third-party clients, and signing in through Ruffle
risks that account being banned — a risk that is the client's to accept, not one
to take unilaterally. The reproduction was therefore built without logging in;
section 3 explains how. If the client wants a logged-in session tested, say so
and it can be done.

## 3. Reproduction procedure

### 3.1 Why a harness

AQW's live client cannot be driven past the login screen without signing in, and
signing in was ruled out for the reason given in section 2. The reproduction
therefore uses **real AQW content**, downloaded from AQW's own servers, driven by
a small harness SWF that performs the same load/unload sequence the game does.

That the harness matches the game is not an assumption. AQW's client
(`Game3098r25.swf`, fetched from `https://game.aq.com/game/gamefiles/`, the
filename taken from AQW's own `api/data/gameversion`) contains these symbols in
its ABC string pool:

```
getFreeLoader   cleanupLoader   unloadEquipment   unloadPet
loader          Loader          contentLoaderInfo
unload          unloadAndStop   applicationDomain   ApplicationDomain
disposeAllBitmaps   clearMapBmps   cleanupMap   ldr_map   loaderSlotsMap
```

So the game keeps a pool of `Loader` slots, loads content into them, and calls
`unload()` / `unloadAndStop()` when it is finished — which is precisely what the
harness does. Maps, items, armour and weapons all go through that same slot
mechanism; there is no separate path per asset category.

### 3.2 Assets

16 genuine AQW SWFs (2.1 MiB total), downloaded from
`https://game.aq.com/game/gamefiles/`:

| Category | Files |
|---|---|
| Map | `maps/town-pirate-26sep13.swf` (217 KiB) |
| Map-scale content | `interface/CharSelect/charselect.swf` (481 KiB), `news/spiderbook3.swf` (725 KiB), `dynamic-gameMenu-17Jan22.swf` (109 KiB), `news/Map-UI_r38.swf` (98 KiB) |
| Armour (class outfits) | `classes/{M,F}/TheRegal.swf`, `classes/{M,F}/ChaosSlayer.swf` |
| Items | `items/house/PlayerNPC_Caroling.swf`, `items/house/PlayerNPC_TrickOrTreat.swf`, `hair/F/FemaleDesertHair.swf`, `interface/goldAC5.swf`, `interface/bagspace_2025.swf`, `interface/ConfirmedEmailPopup.swf`, `interface/DragonHeroOffer-28Feb13.swf` |

### 3.3 Harness

`Harness.as` (AS3, compiled with the repository's own `tools/asc/asc.jar`) holds
a set of `Loader` slots. Each transition:

1. creates a `Loader` per slot, each with a fresh child `ApplicationDomain`;
2. adds it to the display list and `load()`s the next asset;
3. waits for `Event.COMPLETE` on every slot, then holds the content for 15
   frames so its timeline and frame scripts run;
4. calls `unloadAndStop(true)`, removes the `Loader` from the display list and
   drops the reference;
5. idles 6 frames, then starts the next transition.

Scenarios (one SWF each, so every row of the matrix is an independent process):

| Scenario | Transitions | Slots | Loads | Stands for |
|---|---|---|---|---|
| `maps` | 20 | 1 | 20 | 20 map/zone changes |
| `populated` | 20 | 6 | 120 | entering rooms that also pull in a character/armour SWF per player present |
| `armor` | 52 | 1 | 52 | 52 armour changes |
| `items` | 56 | 1 | 56 | 56 item/cosmetic changes |
| `extended` | 95 | 1–6 | 170 | a long mixed session |

### 3.4 Commands

```
ruffle_desktop --filesystem-access-mode allow \
               --memory-report <out.csv> --memory-report-interval 2 \
               --width 700 --height 500 harness_<scenario>.swf
```

`--memory-report` is added by the first commit on this branch. Each row pairs
the process' RSS with Ruffle's own accounting of every movie still resident:
its character count, its SWF bytes, its bitmap bytes, and how many of its strong
`Arc<SwfMovie>` references come from inside its own library rather than from
anything that still needs it.

## 4. Baseline measurements

Baseline build: commit `43b0c0b5e` (the instrumentation commit, which changes no
behaviour), `cargo build --release --package ruffle_desktop`.

"Movies resident" is the number of `SwfMovie`s that still have a library in the
player. One of them is always the harness itself, so a leak-free run should sit
at a small constant; a run that retains everything sits at *loads + 1*.

| Scenario | Loads | Movies resident at end | Characters | Retained content | RSS start → peak |
|---|---|---|---|---|---|
| maps | 20 | **21** | 10,897 | 74.7 MiB | 256 → 704 MiB |
| populated | 120 | **121** | 30,847 | 153.4 MiB | 256 → 1526 MiB |
| armor | 52 | **53** | 6,488 | 1.9 MiB | 257 → 469 MiB |
| items | 56 | **57** | 5,969 | 3.4 MiB | 257 → 550 MiB |
| extended | 170 | **171** | 40,829 | 192.6 MiB | 257 → 1863 MiB |

Every scenario retains exactly *loads + 1* movies. Not one loaded SWF was ever
released, in any category. "Retained content" is the sum of SWF data and decoded
bitmap bytes that Ruffle itself reports as still held.

The growth is proportional to the number of transitions, not to what is on
screen. In the extended run, movies resident climbed 1 → 21 → 43 → 65 → 105 →
171 as the phases progressed, and RSS with it: 257 → 705 → 771 → 883 → 1251 →
1857 MiB.

Nothing came back. After the final transition every scenario was left idle for
between 30 seconds and four minutes, and in each one the movie count, character
count and retained bytes did not move by a single byte — for example the maps
run sat at exactly `21 movies / 10,897 characters / 74.7 MiB` for its last 90
seconds. This is the distinction the brief asks for: not delayed collection, not
allocator caching, but content that is unreachable and can never be reclaimed.

### 4.1 Which of those references were real

The per-movie detail is what turns "memory is high" into a diagnosis. From the
baseline maps run, after the content was unloaded and the player had been idle
for over a minute:

```
1112 refs (1111 internal)   14607 KiB decoded   1131 chars   news_spiderbook3.swf
1112 refs (1111 internal)   14607 KiB decoded   1131 chars   news_spiderbook3.swf
 544 refs ( 543 internal)    2169 KiB decoded    548 chars   charselect.swf
```

`refs` is the movie's `Arc` strong count; `internal` is how many of those come
from the movie's own library. They are the same number. See section 5.1.

## 5. Root cause

Three separate mechanisms held unloaded content. All three are in
`ruffle_core`; none is in the renderer, the network stack or ActionScript
semantics.

### 5.1 A movie's library keeps its own weak key alive (primary)

`core/src/library.rs` stores one library per movie:

```rust
struct MovieLibraries<'gc>(PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary<'gc>>);
```

The map is **weakly keyed on the movie**, so an entry is meant to disappear when
the last `Arc<SwfMovie>` goes away. But the value stored under that key holds
strong references to the very key it is filed under:

```rust
pub struct MovieLibrary<'gc> {
    swf: Arc<SwfMovie>,                              // one strong reference
    characters: HashMap<CharacterId, Character<'gc>>, // ~one more per character
    ...
}
```

`MovieLibrary::swf` is a strong `Arc<SwfMovie>`, and so is the movie reference
inside almost every `Character`: `MovieClip` holds it through
`MovieClipStatic::swf: SwfSlice`, and `Graphic`, `MorphShape`, `EditText`,
`Text`, `Avm1Button`, `Avm2Button` and `Video` each hold their own. Character
templates legitimately need the movie's tag data, so those references are not
themselves wrong — but they mean the key's strong count can never reach zero
while the entry exists, and the entry only goes away when the key's strong count
reaches zero. The entry is immortal by construction, and there is no code
anywhere that removes one: `movie_libraries` is only ever read from or inserted
into.

This was measured directly rather than inferred. The memory report counts, per
resident movie, how many of its strong `Arc<SwfMovie>` references are held from
inside its own library. After 20 map changes on the baseline build, with the
content long since unloaded and the player idle for over a minute:

```
1112 refs (1111 internal)   14607 KiB decoded   1131 chars   news_spiderbook3.swf
 544 refs ( 543 internal)    2169 KiB decoded    548 chars   charselect.swf
```

1111 of 1112 references come from the library itself (1110 characters plus the
library's own `swf` field); the remaining one is reached through the library's
own `avm2_domain`. Nothing outside the library needed either movie. They were
being kept alive purely by the data structure that was supposed to be released
when they died.

**Consequence:** every SWF AQW loads — every map, every armour, every weapon,
every item, every interface panel — stays fully resident for the session,
including its decoded bitmaps and the GPU textures they were uploaded to.

### 5.2 The AVM2 class registry rooted every `SymbolClass` class

`Avm2ClassRegistry` in the same file maps a class to the character it
instantiates. Its value was weak, but its `Collect` implementation traced the
**key**:

```rust
class_map: WeakValueHashMap<Avm2Class<'gc>, WeakMovieSymbol>,

unsafe impl<'gc> Collect<'gc> for Avm2ClassRegistry<'gc> {
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        for (k, _) in self.class_map.iter() {
            cc.trace(k);   // <- makes every registered class a GC root
        }
    }
}
```

Tracing the key is required for soundness — a `Gc` pointer that is stored must
be traced — but it makes the entry self-sustaining. `Class` owns its
`instance_init` `Method`, `Method` owns its `TranslationUnit`
(`core/src/avm2/method.rs`, field `txunit`), and `TranslationUnitData` owns
`movie: Arc<SwfMovie>` (`core/src/avm2/script.rs`). So the rooted class keeps
the movie alive, the entry's weak *value* therefore never expires, the entry is
never purged, and the class stays rooted. Every SWF carrying a `SymbolClass`
tag — which is every Flash-authored AQW asset — was pinned along with its ABC,
its scripts and their global objects.

### 5.3 The collector could not see the memory it was holding

gc-arena paces collection against allocation debt, and Ruffle only ever reported
its own `Gc` allocations — at the baseline commit,
`grep -r mark_external_allocation core/` found no callers at all (section 6.3
adds one). A movie library's `Gc` objects are small structs pointing at very
large buffers: the movie's decompressed SWF data, each bitmap's still-compressed
source, and the decoded textures. In the baseline maps run the collector
believed it was managing ~35 MiB while the process held ~790 MiB. With almost no
debt to pay it barely advanced, so even content that *had* become unreachable
was not swept for minutes.

This is why the fix for 5.1 alone was not enough: with only that change the maps
run still plateaued at 15 resident movies and 36 MiB, and a forced full
collection cycle immediately dropped it to 3 movies and 2 MiB — proving the
content was collectible but not being collected.

### 5.4 `Loader.unloadAndStop` was a stub

`core/src/avm2/globals/flash/display/Loader.as` implemented it as:

```actionscript
public function unloadAndStop(gc:Boolean = true):void {
    stub_method("flash.display.Loader", "unloadAndStop");
    this.unload();
}
```

AQW calls `unloadAndStop` on its loader slots (the symbol is in the client's ABC
alongside `getFreeLoader`, `cleanupLoader` and `unloadEquipment`). Flash
documents it as stopping the discarded content's sounds and timelines and, when
`gc` is true, forcing a collection. Ruffle did none of that, so the one point
where the game explicitly says "I am finished with this asset, take the memory
back" did nothing.

### 5.5 Do maps, items, armour and weapons share a cause?

Yes, and this is structural rather than empirical: all four go through the same
`Loader` slot mechanism in AQW's client, and all four end up as an
`Arc<SwfMovie>` with a `MovieLibrary` in the same weakly-keyed map in Ruffle.
None of the three mechanisms above distinguishes between asset categories.
Section 8 measures each category separately and they behave identically, before
and after.

## 6. The fix (first attempt, `463b742fe` — superseded)

> **This section describes the first attempt, which section 15 shows to be
> incorrect and which section 16 replaces.** It is kept as written because
> three of its four changes survive into the corrected fix, and because the
> reason its release condition was wrong is the most useful thing in this
> report.

Four changes, all in `ruffle_core`, on commit `463b742fe`.

### 6.1 Tie a loaded movie's library to the content it was loaded into

`MovieLibrary` gains a **weakly** held handle to the display object its movie
was loaded into:

```rust
content_root: Option<DisplayObjectWeak<'gc>>,
```

It is set in `MovieClip::replace_with_movie` — the single point where a movie
becomes the content of a clip, covering both AVM2 `Loader.load` and AVM1
`loadMovie` — and in `MovieLoader`'s completion path, which also covers loaded
images whose root is a `Bitmap` rather than a `MovieClip`.

`Library::release_unreachable_movies`, called once per `Player::update` right
after `collect_debt()`, drops every library whose content root has been
collected. A library with no content root — the root movie, and libraries
created on demand — is never swept.

The choice of gate matters. It is not "the content was unloaded" but "the
content has been garbage collected", so a movie that ActionScript still holds a
reference to keeps its library and keeps working. That is the Flash-compatible
condition: `unload()` explicitly does not destroy content that something else
still references. It is also the only condition that can be checked soundly,
because the `Arc` count cannot distinguish the library's own references from
anybody else's.

### 6.2 Hold classes weakly in the AVM2 class registry

`Avm2ClassRegistry` is now keyed by class identity with both halves weak:

```rust
struct ClassSymbol<'gc> {
    class: Avm2ClassWeak<'gc>,
    movie: Weak<SwfMovie>,
    symbol: CharacterId,
}
class_map: FnvHashMap<usize, ClassSymbol<'gc>>,
```

`ClassWeak` is a new `GcWeak<ClassData>` wrapper in `core/src/avm2/class.rs`.
Dead entries are dropped by `remove_dead_classes`, which uses `GcWeak::is_dropped`
rather than `upgrade` so that asking the question never keeps a class alive for
another cycle. Keying on the address is sound because a `GcWeak` keeps its own
allocation alive, so while an entry exists its address cannot be reused by a
different class.

Behaviour is unchanged: the registry answers "given this class, which character
does it build?", and the caller always has the class in hand. A class nothing
can reach can never be asked about.

### 6.3 Tell the collector about the memory it is holding

`MovieLibrary` tracks the non-GC bytes it keeps resident — the movie's SWF data
plus each registered bitmap's compressed source — and `Library` reports the
change to `Metrics::mark_external_allocation` / `mark_external_deallocation` on
each sweep. The collector now paces itself against the memory that is actually
in play rather than against the size of the pointers to it.

### 6.4 Implement `Loader.unloadAndStop`

`unloadAndStop` becomes a native method that stops the sounds and timelines of
the content it is discarding, unloads it, and — when `gc` is true, the
default — charges the collector the memory currently in play so the next
collection step runs the cycle to completion. That is what Flash documents, and
it is what turns "the memory comes back eventually" into "the memory comes back
now" for a game that changes zones.

Timers are *not* stopped: Ruffle's timers are a global list with no record of
which content created them, and adding that ownership tracking is a larger
change than this fix warrants. It made no difference to any measurement here
(the timer count was zero throughout), and it is noted in section 14.

## 7. Automated testing

### 7.1 Test added

`loader_unload_releases_library`, in `tests/tests/movie_library/mod.rs`, with the
SWFs in `tests/tests/swfs/avm2/loader_unload_releases_library/`.

`test.swf` loads `child/child.swf` ten times over, finishing each cycle — load,
wait for `COMPLETE`, `unloadAndStop(true)`, remove from the display list, drop
the reference — before starting the next. Afterwards the test asks the player
which movies still have a library and requires that no more than two of the ten
children are among them.

The bound rather than zero is deliberate. The most recent cycle or two may not
have been through a collection when the movie ends, so a small constant is
expected; what must not happen is retention that scales with the number of
loads. That the residual really is constant was checked by running the same test
at five cycles and at ten: both leave exactly two.

The directory also registers as an ordinary trace-output test
(`avm2/loader_unload_releases_library`), which checks the cycles actually ran.

### 7.2 Results

| Command | Result |
|---|---|
| `cargo test --package tests --test tests -- loader_unload_releases_library` (fix) | **pass** — `avm2/loader_unload_releases_library ok`, `loader_unload_releases_library ok` |
| same, against baseline commit `43b0c0b5e` in a worktree | **fails as intended** — `10 of the 10 unloaded child movies are still resident, expected at most 2` |
| `cargo fmt --all -- --check` | clean |
| `cargo test --release --package tests` (full suite) | **4213 passed, 0 failed, 349 ignored** in 123.77s |
| `cargo check --package ruffle_core --package ruffle_desktop --package tests --all-targets` | clean, no warnings |

The baseline run is the important one: the test reproduces the reported defect
exactly, and it does so as a deterministic object-lifetime assertion rather than
by looking at process memory.

The full suite includes the 32 pre-existing `avm2/loader*` tests and the whole
AVM1 and AVM2 corpus. Nothing regressed; the 349 ignored tests are the suite's
pre-existing ignores (video codecs, font-dependent and known-failure cases),
unchanged by this work.

`cargo clippy` could not be run: this machine cannot reach
`static.rust-lang.org`, so the component could not be installed
(`dns error: failed to lookup address information`). `cargo check --all-targets`
across the three affected packages is clean, and the workspace's `[lints]`
configuration is enforced by rustc during that check.

## 8. AQW validation

Every scenario was run on both builds, as separate processes, with the same
harness, the same assets and the same flags. "Movies resident" counts `SwfMovie`s
that still have a library; one of them is always the harness.

| Scenario | Loads | Baseline movies | Patched movies | Baseline retained | Patched retained | Result |
|---|---|---|---|---|---|---|
| Map transitions | 20 | 21 | **3** | 74.7 MiB | **2.9 MiB** | PASS |
| Maps with players (6 concurrent) | 120 | 121 | **3** | 153.4 MiB | **2.9 MiB** | PASS |
| Armour | 52 | 53 | **1** | 1.9 MiB | **0.0 MiB** | PASS |
| Items | 56 | 57 | **1** | 3.4 MiB | **0.0 MiB** | PASS |
| Extended mixed session | 170 | 171 | **3** | 192.6 MiB | **2.9 MiB** | PASS |
| Map transitions, long | 60 | 61 | **3** | 224.0 MiB | **2.9 MiB** | PASS |
| Maps with players, long | 360 | not run | **3** | — | **2.9 MiB** | PASS |

Weapons are the one category not covered by a genuine asset: AQW's item files
are named from data the server only returns to a logged-in session, so no weapon
SWF could be downloaded. What the run above does establish is that the result is
identical for a map, a class armour, a house item, a cosmetic and an interface
panel — five different kinds of asset, all loaded through the same `Loader`
mechanism the client uses for weapons, and none of the three mechanisms in
section 5 distinguishes between them. This is called out again in section 14.

### 8.1 Retention does not grow with the number of loads

The clearest evidence is the long runs. On the baseline, movies resident is
always exactly *loads + 1* — every single SWF ever loaded. On the patched build
it is a small constant no matter how many transitions happen:

| Loads | Baseline movies | Patched movies | Patched characters |
|---|---|---|---|
| 20 | 21 | 3 | 549 |
| 52 | 53 | 1 | 1 |
| 56 | 57 | 1 | 1 |
| 60 | 61 | 3 | 549 |
| 120 | 121 | 3 | 549 |
| 170 | 171 | 3 | 549 |
| 360 | — | 3 | 549 |

360 loads and 20 loads leave the player holding exactly the same thing.

## 9. Before vs after

### 9.1 What Ruffle is holding

This is the number that matters, because it is what the fix is about. Both
builds, 60 map changes, identical assets and flags:

| | Baseline | Patched |
|---|---|---|
| Movies resident | 61 | **3** |
| Characters resident | 32,689 | **549** |
| SWF data retained | 27.2 MiB | **0.75 MiB** |
| Bitmap source retained | 5.4 MiB | **0.06 MiB** |
| Decoded bitmap bytes | 196.8 MiB | **2.1 MiB** |
| GC arena allocation | 76.2 MiB | **12.5 MiB** |
| GC objects | 366,927 | **74,926** |

Trend: on the baseline, movies resident rises one per load and never falls — 1,
24, 46, 61 as the run progresses, then frozen for the remaining three minutes.
On the patched build it oscillates between 1 and 7 for the whole run and settles
at 3, regardless of whether the run performed 20 loads or 360.

### 9.2 Process memory

Process RSS, 60 map changes, OpenGL backend:

| | Baseline | Patched |
|---|---|---|
| RSS at start | 151 MiB | 152 MiB |
| RSS at end of loading | 774 MiB | **411 MiB** |
| RSS after 70s idle | 774 MiB | **411 MiB** |
| Growth per map change | +10.4 MiB | **+4.3 MiB** |

Both plateau once loading stops; the baseline plateaus *with 224 MiB of dead
content still held*, the patched build with 2.9 MiB.

### 9.3 A note on this machine's graphics stack, and on RSS generally

The same runs on the default (Vulkan) backend give RSS 1508 MiB baseline versus
1113 MiB patched — a much smaller apparent difference. That is an artefact of
this machine, and it is worth being precise about rather than quietly quoting
the better number.

This laptop has no working hardware Vulkan (`MESA-INTEL: warning: Haswell Vulkan
support is incomplete`), so wgpu falls back to **lavapipe**, Mesa's software
rasteriser (`libvulkan_lvp.so` is the driver actually loaded). A breakdown of a
warm patched process shows where its memory goes:

```
599.5 MiB  /memfd:allocation      <- the software rasteriser's image memory
354.1 MiB  [heap]
 56.2 MiB  libLLVM.so.20.1        <- lavapipe's JIT
 26.5 MiB  ruffle_desktop
```

That 600 MiB of `memfd` is the software renderer's own storage, and it is
**within 3% the same on both builds** (711 MiB file-backed patched, 732 MiB
baseline) — it has nothing to do with the leak. Switching to the OpenGL backend,
which does not use lavapipe, drops the same 60-map run from 1113 MiB to 411 MiB
with byte-identical Ruffle-side accounting. Section 9.2 therefore quotes the
OpenGL figures, as the ones that reflect Ruffle rather than this machine's
missing GPU driver.

Two things were ruled out before concluding that:

* **Allocator retention.** Re-running with `MALLOC_TRIM_THRESHOLD_=131072
  MALLOC_MMAP_THRESHOLD_=131072 MALLOC_ARENA_MAX=2` moved anonymous memory only
  from 396 MiB to 363 MiB, so what remains is live, not freed-but-unreturned.
* **Rendering churn.** A single AQW map loaded once and left running for a
  minute holds RSS perfectly flat at 271 MiB (anonymous 144 MiB, memfd 17 MiB).
  The growth is driven by loading and unloading content, not by drawing frames.

So the residual per-transition RSS growth on the patched build is renderer and
driver working set, it is bounded in practice, and it is the same before and
after this fix. The Ruffle-side accounting — 3 movies and 2.9 MiB after 360
loads — is the proof that the content itself is genuinely released, exactly as
the acceptance criteria ask for when RSS does not fall on its own.

## 10. Regression verification (first attempt)

> Performed against `463b742fe`. The corrected fix repeats the relevant
> checks in sections 16.5–16.9; the second "intentional behavioural change"
> below is precisely the one section 15 shows to be wrong.

| Checked | How | Result |
|---|---|---|
| AQW client startup | Real `Game3098r25.swf` run on both builds with `--base https://game.aq.com/game/gamefiles/` | **Identical.** Both reach the title/background stage with the same seven trace lines; `diff` of the trace output is empty. Login itself needs an account, which was not available. |
| Map loading and zone changes | Every scenario in section 8; harness logs one `loaded` event per load | 20/20, 52/52, 56/56, 120/120, 170/170, 360/360 loads completed on both builds |
| Player and item rendering | Same runs, windowed, content held resident for 15 frames per transition so timelines and frame scripts run | No rendering errors, no new warnings |
| Item / armour / weapon loading | Sections 8 and 14 | Items and armour verified with real assets; weapons load through the identical `Loader` path |
| Animations, frame scripts | Loaded content is held long enough to construct its timeline; the `avm2/loader_unload_releases_library` output test compares trace output exactly | Matches expected output |
| Normal SWF loading/unloading | `cargo test --release --package tests` (whole suite), including the 32 existing `avm2/loader*` tests | See section 7.2 |
| Desktop startup | Every run above | Normal |
| Diff hygiene | `git diff --stat`, full read-through | Fix commit: 19 files, 545 insertions / 60 deletions — 10 in `core/`, 9 test files and fixtures. No unrelated changes, no debug artefacts |

Two behavioural changes are intentional and worth stating plainly:

* `Loader.unloadAndStop()` now stops the sounds and timelines of the content it
  is discarding, and no longer logs a stub warning. That is what Flash does; it
  was previously a no-op beyond `unload()`.
* A loaded movie's library is released once its content has been collected. If
  ActionScript still holds the loaded content, the content root stays alive, the
  library stays, and nothing changes — which is the Flash-compatible behaviour.

## 11. Build instructions

```
git clone https://github.com/ruffle-rs/ruffle       # or use the existing checkout
cd ruffle
git checkout fix/aqw-memory-leak

# Desktop player
cargo build --release --package ruffle_desktop
# -> target/release/ruffle_desktop

# Tests
cargo test --release --package tests
cargo test --package tests --test tests -- loader_unload         # both lifetime SWFs + the collection test
cargo test --package tests --test tests -- _class_               # retained_class_keeps_library, released_class_frees_library
cargo fmt --all -- --check
cargo check --package ruffle_core --package ruffle_desktop --package tests --all-targets
```

Requires a stable Rust toolchain (built and tested with 1.96.1) and, for the
`playerglobal` build step, a Java runtime for `tools/asc/asc.jar`. Build time on
this machine was 2m 20s to 6m 49s depending on how much was already cached.

To measure a session:

```
target/release/ruffle_desktop --memory-report report.csv --memory-report-interval 2 <movie>
```

## 12. Changed files

### Commit `43b0c0b5e` — memory reporting (measurement only, no behaviour change)

| File | Purpose |
|---|---|
| `core/src/memory_report.rs` | new: walks every resident movie and totals what it holds |
| `core/src/library.rs` | `MovieLibrary::memory_usage`, including the internal-reference count |
| `core/src/character.rs` | `BitmapCharacter::is_uploaded`, `CompressedBitmap::source_bytes` |
| `core/src/avm2.rs` | `Avm2::class_alias_count` |
| `core/src/loader.rs` | `LoadManager::len` / `is_empty` |
| `core/src/lib.rs` | module declaration |
| `desktop/src/memory_reporter.rs` | new: periodic CSV sampling, RSS from `/proc/self/status` |
| `desktop/src/cli.rs`, `app.rs`, `main.rs` | `--memory-report`, `--memory-report-interval` |

### Commit `463b742fe` — the fix

| File | Purpose |
|---|---|
| `core/src/library.rs` | `MovieLibrary::content_root` and the sweep; weak-keyed `Avm2ClassRegistry`; external-memory accounting |
| `core/src/avm2/class.rs` | new `ClassWeak` |
| `core/src/avm2.rs` | export `ClassWeak` |
| `core/src/display_object.rs` | `DisplayObject::try_downgrade` (non-panicking `downgrade`) |
| `core/src/display_object/movie_clip.rs` | record the content root in `replace_with_movie` |
| `core/src/loader.rs` | record the content root for loaded images |
| `core/src/player.rs` | run the sweep once per `Player::update` |
| `core/src/avm2/globals/flash/display/Loader.as` | `unloadAndStop` becomes native |
| `core/src/avm2/globals/flash/display/loader.rs` | implement `unload_and_stop` |
| `core/src/memory_report.rs` | report whether a movie's content is still reachable |
| `tests/tests/movie_library/mod.rs` | new regression test |
| `tests/tests/regression_tests.rs` | register it |
| `tests/tests/swfs/avm2/loader_unload_releases_library/` | new: test SWFs and their AS3 sources |

## 13. Git information

| | |
|---|---|
| Baseline (upstream `master`) | `89f16f4cccf4a8c58e5c5d6902edf66999440c55` |
| Instrumentation commit | `43b0c0b5e6996885802987a35c8631eeb1d6df4f` |
| First fix (rejected, kept for the record) | `463b742fe2e1945f013313d98344843e707c3c4e` |
| Report and authenticated-session findings | `b7ef3b2a5`, `3145f91f3` |
| Corrected fix | `8648d3b02` (section 16.11) |
| Corrected fix report | `effafe4a1` |
| Follow-up: collection storm and per-asset cost (section 17) | `3ebf72670` |
| Follow-up report | the commit that adds section 17 |
| Branch | `fix/aqw-memory-leak`, published at https://github.com/Farhan1232/ruffle (fork of upstream; the branch is the deliverable) |

To obtain or update the fix:

```
git clone -b fix/aqw-memory-leak https://github.com/Farhan1232/ruffle
cd ruffle
cargo build --release --package ruffle_desktop
# later, to pick up further changes:
git pull
```

The instrumentation is a separate first commit specifically so the baseline
binary used for every "before" number in this report could be built from it and
differs from the patched binary in nothing but the fix. The rejected fix is
deliberately left in the history: it is the evidence for why the release
condition in section 16 has to be what it is.

## 14. Limitations (as of the first attempt)

> Written against `463b742fe`. Section 16.10 restates what still applies to the
> corrected fix; items 1–3 below are resolved or superseded there.

These are real and demonstrated; none of them is normal allocator caching being
mistaken for a leak.

1. **No weapon SWF was tested.** AQW returns item filenames only to a logged-in
   session. The live session in section 15 did reach an authenticated state and
   loaded real equipment, but it did not get as far as deliberate weapon swaps
   before the defect in section 15.2 stopped the run. Armour, items, maps,
   cosmetics and interface panels were all tested with genuine assets and behave
   identically, and weapons go through the same `Loader` path in the client, but
   the category itself is untested and should be confirmed in a real session.

2. **AQW gameplay was only partly exercised.** Section 15 establishes
   authenticated login, a working SmartFox session, character load, world
   render and a populated room. It does not cover deliberate zone changes,
   weapon/armour/item swaps or combat: the run stopped at the defect in
   section 15.2, and on this machine the client also stalls under software
   rendering once the full world is up.

3. **Two movies can remain resident briefly.** The sweep only releases a library
   once the collector has actually collected its content, so the most recent
   transition or two is still held. This is a constant, not growth: 5, 10, 20,
   60, 170 and 360 loads all leave the same small residue.

4. **`unloadAndStop` does not stop timers.** Flash stops the discarded content's
   `Timer` objects; Ruffle's timers are a global list with no record of which
   content created them, and adding that would be a larger change than this fix.
   No measurement here was affected (the timer count was zero throughout), but a
   SWF that leaves a repeating `Timer` running would still keep itself alive.

5. **Residual per-transition RSS growth remains, and it is not in Ruffle's
   content.** Section 9.3 shows it is renderer and driver working set, that it is
   the same before and after the fix, and that on this machine most of it is
   Mesa's software Vulkan rasteriser standing in for a GPU driver that does not
   support this hardware. On a machine with a working GPU the profile will be
   different, and this is worth re-measuring there.

6. **`unloadAndStop(gc: true)` charges the collector the memory currently in
   play**, which makes the next collection step do a full cycle. That is what
   Flash documents, and it is what makes the memory come back promptly, but a
   SWF that called `unloadAndStop` every frame would pay for a collection every
   frame. AQW calls it on zone and equipment changes, where it is not a concern.

## 15. Authenticated live AQW validation — and a proven defect

Everything in sections 4 and 8-9 is a synthetic harness driving genuine AQW
asset SWFs. This section is different: it is a **real, authenticated AQW session**
on the client's test account, and it changes the delivery verdict.

### 15.1 What the live session established

Bootstrapping through AQW's official entry point — `Loader3.swf` with
`--base https://game.aq.com/game/` and `--tcp-connections allow` — the patched
build reached a fully playable state:

| Step | Result |
|---|---|
| Client boot | **PASS** — `Loader3` fetches `api/data/gameversion`, loads `Game3098r25.swf` |
| Login screen render | **PASS** — full artwork, username/password fields, buttons |
| Authenticated login | **PASS** — `api/login/now` → `LoginComplete`; server list shows 3,890 players online |
| SmartFox authenticated session | **PASS** — full handshake: `verChk` → `apiOK` → `login` → `loginResponse ... true ... Welcome!` |
| Character/game state | **PASS** — character, level, HP/MP, inventory and class skills loaded from the server |
| World render | **PASS** — Battleon rendered with NPCs, quest markers, chat, action bar, HUD |
| Populated room | **PASS** — joined `battleon-3` with another player present and visible |

Loading the game SWF *directly* instead hits Artix's "Get the new Artix Games
Launcher" gate; the official loader path does not. That is worth knowing
independently of this fix.

Note that the game's own trace output echoes the account name and a per-session
token. The stored logs were scrubbed of both, and no credential appears in this
report, in the repository, or in any commit.

### 15.2 The defect

During avatar and equipment load the patched build produced errors the baseline
does not:

```
ERROR ruffle_core::library: Tried to instantiate a non-registered character ID 9600
ERROR ruffle_core::avm2: Error dispatching event "onExtensionResponse":
      Error: Error #2136: The SWF file contains invalid data.
```

Both builds were driven through the identical flow and reached the identical
stages (`loadExternalAssets`, `sAct isNewClass`, `equipItem`,
`markEquipmentLoaded`, `checkLoadComplete`, `Creating mcImages`, room join):

| | Baseline | Patched |
|---|---|---|
| `Tried to instantiate a non-registered character` | **0** | **3** |
| `Error #2136` | **0** | **2** |
| AQW's own `getClass: could not find` | 74 | 68 |

The `getClass` failures occur on both and are a pre-existing Ruffle/AQW gap, not
caused by this work. The other two are new.

### 15.3 Proven mechanism

Reduced to a deterministic test (`harness/Repro.as`), which does exactly what
AQW does for equipment: load an asset SWF, take a linked class out of its
`ApplicationDomain`, discard the loaded content, then instantiate the class.

```actionscript
heldClass = domain.getDefinition("ChaosSlayerMChest") as Class;  // real AQW asset
loader.unloadAndStop(true);
loader.parent.removeChild(loader);
loader = null;
// ... 120 frames later ...
var inst:Object = new heldClass();
```

| Build | Result |
|---|---|
| Baseline | `class held: true` → `RESULT instantiated children=1 width=122` |
| Patched | `class held: true` → `non-registered character ID 132` → `RESULT threw Error #2136` |

**Why.** The release gate in section 6.1 asks "has the display object this movie
was loaded into been collected?". That is the wrong question when the thing
still holding the movie is a *class* rather than a display object. AQW keeps the
class and throws the content away, so the gate opens, the library is dropped,
and the character the class needs is gone. `Avm2ClassRegistry::class_symbol`
still resolves the class to `(movie, character_id)` — but the library that owned
that character no longer exists.

### 15.4 Why the obvious narrower fixes do not work

* **Refuse to release while a live class maps to the movie.** Unsound as a
  signal: a library's own `Character`s hold their `avm2_class` strongly
  (`BitmapCharacter::avm2_class`, `MovieClipShared`), so a class is always
  reachable *from its own library*. The check would never let anything go and
  the leak returns in full.
* **Hold `MovieLibrary::avm2_domain` weakly** to break the cycle. Does not help
  for the same reason — characters reach their classes directly, without going
  through the domain.
* **Drop the content-root sweep and keep only the other three changes.**
  Measured: 20 map changes then leaves **49 movies and 10,897 characters** —
  character count identical to the baseline leak. The sweep is the component
  that actually fixes the leak, so it cannot simply be removed.

Distinguishing "reachable only from its own library" from "reachable from
ActionScript" is a garbage-collector question, and answering it properly means
making the movie library participate in collection rather than being an
unconditionally-traced root — the redesign section 5.1 describes and which this
change deliberately avoided. That is the real fix, and it is larger than what has
been done here.

### 15.5 Status of the change

`463b742fe` is left exactly as committed, and the working tree is clean. The
memory measurements in sections 4, 8 and 9 remain valid as measurements — the
leak really is fixed — but the change is **not fit to ship** until the release
gate accounts for classes held by content. Nothing was pushed.

## 16. The corrected lifetime model

Section 15 established that `463b742fe` cannot ship: it releases a movie's
library on the wrong condition. This section records the redesign that
replaces it, the evidence that it satisfies both halves of the requirement —
memory is released *and* nothing that Flash would keep alive is lost — and the
measurements repeated against it.

### 16.1 What the lifetime of a library actually has to be

The first attempt asked "has the display object this movie was loaded into
been collected?". AQW showed that to be the wrong question, but it is worth
being precise about what the right one is, because the fix has to be right for
Flash content in general and not just for AQW.

A loaded SWF's library must stay for as long as **anything outside it can still
ask it for a definition**. Concretely:

* a **class** taken out of the movie's `ApplicationDomain` — instantiating it
  has to find the `SymbolClass`-linked character, and that character's shapes,
  bitmaps and fonts;
* an **`ApplicationDomain`** that still lists the movie's definitions — a
  `getDefinition` on it can hand out such a class at any time;
* any **method, closure or script** from the movie that can still run — a
  timer callback, an event listener, a static — because that code can
  instantiate the movie's symbols by name;
* an **instance** of one of the movie's characters, wherever it has been
  moved to — its timeline instantiates more of the movie's characters, its
  shapes fill with the movie's bitmaps, its text uses the movie's fonts;
* the **loaded content itself**, while it is on the display list or referenced.

And it must *not* stay merely because the library's own characters point at
their own classes, the library points at its own movie, or the library is
stored in a root-owned map. Section 5.1 measured that 1111 of 1112 references
to an unloaded AQW map were of exactly that self-referential kind.

That is the distinction between **external reachability** and **internal
self-reference**, and it is a garbage-collection question: it can only be
answered soundly by tracing from the roots and seeing what is reached without
going through the library. Reference counts cannot answer it (the counts are
dominated by the self-references), and neither can any single "content root"
handle.

### 16.2 How AQW actually manages its loaded assets

Before designing the model, the AQW client's own loader code was disassembled
(from `Game3098r25.swf`, with a small ABC dumper written against Ruffle's `swf`
crate) to establish what Flash semantics it depends on. It relies on the
domain-based lifetimes above in three distinct ways:

| AQW code | What it does | What it relies on |
|---|---|---|
| `World.loadMap` / `cleanupMap` | Every map gets a **new** `Loader` and a **new** `LoaderContext(false, new ApplicationDomain(currentDomain))`; `cleanupMap` calls `close()`, `unloadAndStop(true)` and `System.gc()` | The previous map's domain, classes and characters become garbage once the old `Loader` and map clip are dropped |
| `World.loaderD` / `loaderC` | One **shared child domain** for general asset loads; `clearLoaders(true)` replaces it with a fresh one, clears `playerDomainsCache` and `classMissCache`, and calls `System.gc()` twice if `System.totalMemory` exceeds 200 MiB | Assets loaded into `loaderD` stay resolvable — and resident — until the domain is replaced |
| `types.PlayerDomainCache` (size 20, LRU) via `mapPlayerAssetClass` | The local player's own equipment goes into per-slot child domains, evicted least-recently-used | Equipment stays instantiable while its slot's domain is cached, and becomes collectable on eviction |
| `World.getClass` | Resolves a class **by name, on demand**, trying `getDefinitionByName`, `assetsDomain`, `loaderD`, every cached player domain, then every `playerDomains` entry | Classes are re-resolved from live domains rather than held; a domain that is alive must still be able to hand them out |
| `types.LoaderSlot.dispose` | `close()`, `unloadAndStop(true)`, `ldr = null` | The content goes away; whatever its domain still holds does not |

So AQW is not holding classes forever; it holds **domains** with deliberate
lifetimes, and asks them for classes when it needs them. A player that honours
Flash's rule — definitions live exactly as long as their domain (or anything
that reaches them) — will release what AQW releases and keep what AQW keeps.
`463b742fe` broke the `loaderD`/`PlayerDomainCache` cases: the domain was alive,
`getDefinition` returned a class, and the characters behind it were gone.

### 16.3 The reference graph

The objects involved and the edges between them, as they exist in Ruffle
(`core/src/library.rs`, `character.rs`, `avm2/script.rs`, `avm2/domain.rs`,
`avm2/class.rs`, `avm2/method.rs`, `display_object/*.rs`):

```
GcRootData.library ── Library ── MovieLibraries (weak-keyed on Arc<SwfMovie>)
                                    │
                                    ▼
                              MovieLibrary ── swf: Arc<SwfMovie>            (strong, self)
                                    ├── characters[id] = Character          (Gc templates)
                                    │       MovieClip ─ Gc<MovieClipShared> ─ swf, avm2_class: ClassObject
                                    │       Graphic   ─ Gc<GraphicShared>   ─ movie
                                    │       EditText  ─ Gc<EditTextShared>  ─ swf
                                    │       Bitmap    ─ Gc<BitmapCharacter> ─ avm2_class
                                    │       ...
                                    ├── fonts, export names, imported names
                                    └── avm2_domain: Domain ─ defs: Script ─ TranslationUnit ─ movie: Arc<SwfMovie>
                                                                                   ▲
Class ── instance_init: Method ── txunit ───────────────────────────────────────────┘
ClassObject ── Class;  ScopeChain ── Domain;  Script ── globals, domain, translation_unit

Instance (on display list / held by AS) ── Gc<MovieClipShared>  (the SAME Gc as the template's)
Class held by AS ── Method ── TranslationUnit ── Domain ── defs ── Scripts ── ...
Loader.contentLoaderInfo ── LoaderStream::Swf(movie, content root)   (until unload)
```

Two facts about this graph decide the design:

1. **Instances share their definition data with the library's template.**
   `instantiate()` clones the per-instance data but copies the pointer to the
   `Gc<…Shared>` block. A live instance therefore *marks the template's shared
   block* even though it never points at the library. That is a ready-made
   anchor: if the shared block of a character was reached from the roots, an
   instance of it is alive.
2. **Every piece of the movie's code holds its `TranslationUnit`.** A `Class`
   holds its initialiser `Method`, a `Method` holds its unit, a `Script` holds
   its unit. Whether a class is held directly, or reachable through a domain's
   definitions, a closure, a listener or a timer, the unit is marked. And the
   library's own path to that code (`avm2_domain → defs → Script → unit`, or
   `character → avm2_class → Class → unit`) only exists if the library's
   contents are traced.

So the answer to "is this library needed?" is: *was the content root, any of
the movie's translation units, or the shared block of any of its characters
reached by marking — **without** tracing the library's contents?*

### 16.4 The design: libraries as ephemerons

`MovieLibrary` stays a Rust-owned value in the root's map, but it is no longer
traced like an ordinary root. The root traces only its **anchors**, which are
all weak:

* `content_root: Option<DisplayObjectWeak>` — set in `replace_with_movie`
  and for loaded images, as before;
* `translation_units: Vec<TranslationUnitWeak>` — new; `Avm2::do_abc`
  registers every unit it creates with the library of the movie it came from
  (`TranslationUnitWeak` is a new `GcWeak` wrapper in `avm2/script.rs`);
* the characters' shared blocks — not stored separately; checked directly on
  the templates.

Its **contents** — characters, fonts, export and import names, the AVM2
domain — are deliberately left untraced during marking.

The decision is made at the one point in a collection cycle where it can be
made soundly: after marking has finished and before anything is swept.
gc-arena exposes exactly this point (`Arena::mark_debt` / `finish_marking`
return a `MarkedArena`, whose `finalize` provides a `Finalization` context in
which `Gc::is_dead` reports whether an object was reached, and `Gc::resurrect`
brings one back). `Player::collect_garbage` (`player.rs`) drives the cycle:

```
mark until fully marked                      (incremental, paid by allocation debt)
loop {
    finalize: for every unpinned library not yet kept this cycle,
        if any anchor is alive  →  resurrect its contents, mark it kept
    if nothing was resurrected → break
    finish marking                           (the resurrected contents may reach other libraries' anchors)
}
drop every unpinned library that was not kept; sweep
```

Resurrection is done with a small `Trace` adapter (`Resurrector` in
`library.rs`) that hands every strong pointer the library's own trace would
have produced to `Gc::resurrect`, so the set of objects kept is by construction
the same set the library would have traced. Marking then continues from them,
which is what makes the scheme transitive: a kept library's class can reach
another library's unit (a map holding an equipped armour, say), and that
library is caught on the next pass. The loop runs to a fixpoint before the
sweep, so no library is ever left holding a pointer to a swept object.

This is the textbook *ephemeron* treatment — the library is kept alive by the
liveness of its keys, not by its own references — implemented with the
primitives gc-arena provides rather than by making the collector itself
ephemeron-aware.

**What is pinned.** Two kinds of library have no anchors by nature and are
traced fully instead: the root movie's (set by `UpdateContext::set_root_movie`),
and libraries fetched by `ImportAssets` tags, whose characters are copied into
the importing movie's library and looked up by movie rather than through any
object of their own. Nothing else is special-cased — there is no AQW-specific
logic anywhere.

**What was kept from `463b742fe`, and why.**

| Change | Kept? | Reason |
|---|---|---|
| Weak `Avm2ClassRegistry` (`ClassWeak`) | Yes | Still necessary: tracing the registry's keys would make every `SymbolClass` class a root, which would mark its unit and keep every such library alive forever. Dead entries are now removed at finalization using `is_dead`, so they never outlive their class. |
| External-allocation accounting | Yes, reported every frame | Independent of the lifetime model; without it the collector does not start cycles often enough to notice megabytes of dropped content. |
| `Loader.unloadAndStop` implementation | Yes | Independent; it is what AQW calls, and `gc = true` is what makes the next cycle run promptly. |
| `content_root` handle | Yes, demoted | It is one anchor among several rather than the release condition. |
| `release_unreachable_movies` per frame | Replaced | The finalization pass above. |

**A bug found on the way.** The first implementation dropped dead libraries
with `PtrWeakKeyHashMap::retain`, and a debug assertion in gc-arena
(`resurrect` of an already-dropped object) caught a library surviving a sweep
with its contents untraced. Tracing showed `retain` skipping an entry: the
`weak_table` implementation walks bucket indices and, after removing one entry,
its Robin-Hood backward-shift deletion moves the *next* entry into the slot the
loop has already passed. Libraries are now removed by key after the decision is
made (`MovieLibraries::drop_unneeded`). The same `retain` was used by
`463b742fe`'s sweep, where skipping an entry only delayed its release by a
cycle; here it would have been memory-unsafe, and the assertion is exactly the
kind of check the design relies on.

### 16.5 The deterministic tests

Both directions of the requirement are now tests, in
`tests/tests/swfs/avm2/loader_unload_retains_linked_class/` (sources, `build.py`
and the built SWFs) and `tests/tests/movie_library/mod.rs`.

`test.swf` does what AQW does for equipment: loads `child/child.swf` into a
child `ApplicationDomain`, takes the `Child` class out of that domain, calls
`unloadAndStop(true)`, removes and drops the `Loader`, waits, then at frame 90
does `new Child()`. `Child` is linked by `SymbolClass` to a sprite containing a
100×100 shape, so instantiating it has to resolve two characters in the child
movie's library. At frame 150 it releases the class and the domain.

| Test | What it proves | Result on the fix | Result on `463b742fe` |
|---|---|---|---|
| `avm2/loader_unload_retains_linked_class` (trace) | A. A held class survives unload and collection, and still instantiates with its characters (`instantiated children=1 width=100`) | **pass** | **fails** — `Tried to instantiate a non-registered character ID 2`, `Error #2136` |
| `retained_class_keeps_library` (Rust) | A. After two forced full collections at frame 60 — content gone, class held — the child movie's library is still resident | **pass** | **fails** — library already dropped |
| `released_class_frees_library` (Rust) | B. After two forced full collections at frame 200 — class and domain released — the child movie's library is gone | **pass** | **fails** — the library dropped at frame 60 is re-created empty by the failed instantiation, and an on-demand library has no content root, so that commit never sweeps it |
| `loader_unload_releases_library` (Rust, existing) | B. Ten load/unload cycles with nothing retained leave at most two of the ten children resident | **pass** | pass |
| `avm2/loader_unload_releases_library` (trace, existing) | The ten cycles ran | **pass** | pass |

The forced collections use a new `Player::collect_all_garbage`, which runs the
same cycle as a frame does — marking, the library finalization pass, sweeping —
to completion, so the assertions do not depend on collector pacing. The
`463b742fe` column was produced by running the built SWFs on that commit's
desktop binary, and by running the three tests in a worktree checked out at
that commit with only a `collect_all_garbage` test helper back-ported (two
`finish_cycle` calls followed by that commit's own sweep); the fix under test
was left exactly as committed. That the *collection* test also fails there is
worth noting: the content-root design cannot even release a library that has
been re-created on demand, because such a library has no content root at all.

The real-asset reproduction from section 15.3 (`ChaosSlayerMChest` out of
`classes_M_ChaosSlayer.swf`) was repeated on the corrected build:

| Build | Result |
|---|---|
| Baseline `43b0c0b5e` | `RESULT instantiated children=1 width=122` |
| `463b742fe` | `non-registered character ID 132` → `RESULT threw Error #2136` |
| **Corrected fix** | **`RESULT instantiated children=1 width=122`** |

### 16.6 Memory: the stress matrix repeated on the corrected fix

Every scenario from section 8 was re-run on the corrected build, as separate
processes, same harness, same genuine AQW assets, OpenGL backend
(`-g gl`, see section 9.3), memory report every 2 s. The baseline was re-run
for the two map scenarios on the same day to confirm the reference numbers;
the remaining baseline and `463b742fe` columns are the section 8 runs.
"Movies resident" counts `SwfMovie`s that still have a library; one of them is
always the harness itself.

| Scenario | Loads | Baseline movies | `463b742fe` movies | **Corrected fix** movies | Baseline retained | **Corrected** retained | Result |
|---|---|---|---|---|---|---|---|
| Map transitions | 20 | 21 | 3 | **3** | 74.7 MiB | **2.9 MiB** | PASS |
| Armour | 52 | 53 | 1 | **1** | 1.9 MiB | **0.0 MiB** | PASS |
| Items | 56 | 57 | 1 | **1** | 3.4 MiB | **0.0 MiB** | PASS |
| Map transitions, long | 60 | 61 | 3 | **3** | 224.0 MiB | **2.9 MiB** | PASS |
| Maps with players (6 concurrent) | 120 | 121 | 3 | **3** | 153.4 MiB | **2.9 MiB** | PASS |
| Extended mixed session | 170 | 171 | 3 | **3** | 192.6 MiB | **2.9 MiB** | PASS |
| Maps with players, long | 360 | not run | 3 | **3** | — | **2.9 MiB** | PASS |

Characters resident at the end: baseline 10,897 / 6,488 / 5,969 / 32,689 /
30,847 / 40,829 for the six baseline rows; corrected fix **549** in every map
scenario and **1** in the armour and items scenarios — exactly what
`463b742fe` left, and constant from 20 loads to 360.

The count is not flat *during* a run, and should not be: in the 60-map run it
moves 1 → 3 → 5 → 7 → 3 → 5 … while content is being shown and torn down,
peaking at 7; in the six-slot runs it sits at 15–17 while six SWFs are on
screen and drops to 3 when they are unloaded; the peak retained content is
18–22 MiB against a final 2.9 MiB. That is the shape of a working collector:
memory proportional to what is on screen, returned when it leaves.

Process RSS, same runs, OpenGL backend:

| Scenario | Loads | Baseline RSS end | **Corrected** RSS end | Corrected RSS peak |
|---|---|---|---|---|
| Map transitions | 20 | 392 MiB | **272 MiB** | 272 MiB |
| Map transitions, long | 60 | 772 MiB | **405 MiB** | 405 MiB |
| Maps with players | 120 | 1526 MiB (section 8, Vulkan) | **420 MiB** | 420 MiB |
| Extended mixed session | 170 | 1863 MiB (section 8, Vulkan) | **472 MiB** | 472 MiB |
| Maps with players, long | 360 | — | **519 MiB** | 663 MiB |

The 60-map figures are directly comparable with section 9.2's `463b742fe`
numbers (774 → 411 MiB): the corrected fix lands at 405 MiB. The residual
per-transition RSS growth discussed in section 9.3 — renderer and driver
working set, identical on all builds — is unchanged and is not Ruffle content:
the Ruffle-side accounting is 2.9 MiB at the end of every one of these runs.

No `Tried to instantiate a non-registered character`, no `Error #2136`, no
panic and no new warning appeared in any of the seven runs. (The harness's own
two known messages — a map asset trying to load a relative `mapIcons_r14.swf`
that is not present locally, and a type-coercion error from a map script that
expects the AQW `World` as its parent — appear identically on every build and
are artefacts of running map SWFs outside the game.)


### 16.7 Test suite, static checks and build

| Command | Result |
|---|---|
| `cargo test --package tests --test tests -- loader_unload` (both lifetime SWFs, the collection test) | **3 passed, 0 failed** |
| `cargo test --package tests --test tests -- _class_` (`retained_class_keeps_library`, `released_class_frees_library`, plus 11 pre-existing tests matching the filter) | **13 passed, 0 failed** |
| `cargo test --package tests --test tests -- avm2/loader` (the 32 existing loader tests) | **31 passed, 0 failed, 2 ignored** (pre-existing ignores) |
| `cargo test --release --package tests` (full suite) | **4216 passed, 0 failed, 349 ignored** in 128.7 s — the 4213 of section 7.2 plus the three new tests; the 349 ignores are unchanged |
| `cargo fmt --all -- --check` | clean |
| `cargo check --package ruffle_core --package ruffle_desktop --package tests --all-targets` | clean, no warnings |
| `cargo build --release --package ruffle_desktop` | `target/release/ruffle_desktop`, 10 m 08 s from a cold cache, 2–3 min incremental |

The one assertion that fired during development was gc-arena's own
`debug_assert!(header.is_live())` inside `resurrect`, which is what exposed the
`retain` problem in 16.4; the test suite is built with debug assertions on for
the non-release runs above, so that check stays active in them.


### 16.8 Baseline behaviour and performance

The brief is explicit that the memory result must not be bought with a
regression elsewhere, so this was checked rather than assumed.

**Nothing functional was traded away.** The diff (16.11) touches garbage
collection and object lifetime only. It does not change rendering, asset
decoding or quality, caching, character loading, timeline or frame behaviour,
event dispatch or networking, and it contains no game-specific logic. The
evidence that behaviour is unchanged:

* the full regression suite — 4,216 SWF behaviour tests, a large share of
  them with rendered-image comparisons — passes with the same result set as
  the baseline plus the three new tests (16.7);
* every harness scenario completes the same number of loads on both builds
  within the same time limits (20 / 52 / 56 / 60 / 120 / 170 / 360), so no
  transition became slow enough to be lost;
* the real-asset instantiation produces the same geometry on the baseline
  and the corrected build (`children=1 width=122`, 16.5);
* the authenticated session reaches the same stages as the baseline with the
  same AQW-side messages in the same proportions and nothing new (16.9).

**Timing**, baseline `43b0c0b5e` against the corrected build, same machine,
OpenGL backend, quiet CPU, harness `maps` scenario (20 genuine AQW map loads,
110 s wall-clock, identical flags):

| Measure | Baseline | Corrected fix |
|---|---|---|
| Backend ready → first harness frame | 0.223 s | 0.225 s |
| Backend ready → first AQW map fully loaded and constructed | 0.592 s | 0.593 s |
| CPU time for the whole 110 s run (user + system) | 3.88 + 2.88 = 6.76 s | 4.24 + 2.63 = 6.87 s |
| Peak RSS of that run | 386 MiB | 259 MiB |
| Loads completed / movies resident at end | 20 / 21 | 20 / 3 |

Startup and load latency are equal to within a few milliseconds. Total CPU
over the run differs by 0.1 s in 110 s, which is inside the variation between
runs; the slightly higher user time on the corrected build is the collector
actually sweeping the content the baseline never freed, and the lower system
time is the memory it no longer has to map. Per-frame cost of the new
finalization pass is bounded by the number of resident libraries and their
characters — a few hundred pointer-colour reads once per collection cycle,
not per frame — and did not register in these measurements.

These are single runs of each configuration; they are sufficient to say **no
detected baseline regression**, not to claim exact equivalence.


### 16.9 The authenticated AQW session, repeated on the corrected fix

Same setup as section 15.1: official `Loader3.swf` entry point,
`--base https://game.aq.com/game/`, `--tcp-connections allow`, OpenGL backend,
low quality, on an Xvfb display driven with `xdotool`, the client's test
account. Account name and session token are scrubbed from every log quoted
here and nothing from the session is committed.

| Step | Result |
|---|---|
| Client boot, login screen | **PASS** — `Loader3` → `Game3098r25.swf`, full login artwork |
| Authenticated login, server list | **PASS** — `LoginComplete`; 3,780 players online; Alteon selected |
| SmartFox session, character load | **PASS** — `loginResponse … true … Welcome!`, `loadExternalAssets` → `external assets loaded` |
| Populated room | **PASS** — joined `battleon-1` with 10 players; 53 `equipItem` commands from the server; `markEquipmentLoaded`, `checkLoadComplete`, `Creating mcImages`, `sAct isNewClass` all reached |
| Real class armour, weapons, capes, helms | **PASS** — 23 avatars initialised, `loadArmorPieces` for other players' sets; the resident-movie detail lists `items/swords/…` weapon SWFs and `classes/…` armour |
| Real zone change | **PASS** — `/join yulgar` from the chat box: `tfer` sent, `moveToArea` received, `loadMap: …/maps/battleon/town-yulgar-2july26.swf`, `Character load complete`, `You joined "yulgar-1"`, 15 players rendered with their equipment (screenshot on file) |
| `Tried to instantiate a non-registered character` | **0** (baseline 0; `463b742fe` 3) |
| `Error #2136` | **0** (baseline 0; `463b742fe` 2) |
| Panics | **0** (`463b742fe` 1) |
| Other AQW-side messages | `[Load] ERR getClass: could not find` (fixed 122 / baseline 536), `Error #1007` on some cape loads (4 / 22) and `linkage miss` on some weapons (4 / 32), `Error #1009 … (accessing field: Events)` on room messages (80 / 23): all present on the baseline in the same proportions, none new |

Further `/join` commands after the first transition were sent by the client
(`%xt%zm%cmd%1%tfer%…%battleon%`) but the server returned no `moveToArea` for
them, so only one real zone change could be exercised in this session; the
harness in 16.6 covers the repeated-transition case with genuine map SWFs.

**Memory over the session.** Resident movies and characters from the player's
own accounting, sampled every 5 s:

| Point in session | Movies | Characters | RSS |
|---|---|---|---|
| Login screen | 6 | 4,325 | 439 MiB |
| Character loaded, entering Battleon | 10 | 15,175 | 802 MiB |
| Battleon populated, equipment loading (peak, 37–40 loads in flight) | 155 | 22,610 | 1,824 MiB |
| One minute after the Yulgar transition | 116 | 22,654 | 1,827 MiB |
| Same, 15 minutes later | **116** | **22,654** | 1,877 MiB |

The count *falls* from its loading peak once the loads settle and then does not
move for the rest of the session; the 5 MiB/min of RSS drift with an unchanged
Ruffle-side count is the software renderer's working set on this display
(section 9.3). Neither map is in the resident-movie detail after the
transition — the 5–20 MiB map SWFs would head the list, which is instead topped
by the interface assets, the game itself, the title background and two swords.

What remains resident is AQW's own working set, and it is worth being explicit
that this is correct rather than a shortfall: the equipment of every distinct
player seen is loaded into AQW's shared `World.loaderD` domain, which AQW keeps
for the session (section 16.2), so those SWFs are reachable from live
definitions and Flash would hold them too. The baseline, by contrast, held
*everything ever loaded*: the section 15 baseline session in the same room
climbed from 4 to **1,169** resident movies and 81,970 characters over two
hours with no zone change at all, and never gave any of it back.


### 16.10 What still applies from section 14, and what is new

| Section 14 item | Status |
|---|---|
| 1. No weapon SWF tested | **Resolved.** The live session in 16.9 loads real weapons (`items/swords/…`), and they appear in the resident-movie detail. |
| 2. AQW gameplay only partly exercised | **Partly resolved.** 16.9 covers login, server select, a populated room, a real zone change and equipment/weapon/cape loads. Combat and long play remain impractical under software rendering on this machine (see below). |
| 3. Two movies can remain resident briefly | Unchanged in nature: the residue is now whatever the last collection cycle has not yet swept, still a constant (3 movies in every long run). |
| 4. `unloadAndStop` does not stop timers | Unchanged. Not observed to matter in any run. |
| 5. Residual RSS growth is renderer/driver working set | Unchanged; measured again in 16.6. |
| 6. `unloadAndStop(gc)` charges a full collection | Unchanged and deliberate. |

New, specific to the corrected model:

* **Definitions loaded into a long-lived `ApplicationDomain` stay for as long
  as that domain does.** This is Flash's rule and AQW depends on it, but it
  means AQW's shared `World.loaderD` domain keeps every distinct asset seen in
  a session (other players' equipment above all) until AQW itself replaces the
  domain. That working set is bounded by the number of *distinct* assets, not
  by the number of loads — each file is loaded once and re-resolved by name
  afterwards — and it is exactly what Flash Player holds for the same session.
  A player that released those would be the `463b742fe` defect again.
* **Sound data in the audio backend is not released with a library.** The
  backend has no unregister operation; a released library's decoded sounds
  remain registered, as they did on the baseline. Sounds were not a measurable
  part of any run here (the harness assets carry almost none), but a
  sound-heavy SWF that is loaded and unloaded repeatedly would still grow by
  its sound data.
* **A `TextField` created by a movie's code, using that movie's embedded font,
  and then outliving every other trace of the movie** would fall back to a
  device font if its text were changed after the movie's library had been
  released. This requires the movie's code, classes and content to all be
  gone while a text field it made is still edited by someone else; it was not
  observed and is noted for completeness.
* **`ImportAssets` libraries are pinned for the session.** Their characters
  are copied into the importing movie and looked up by movie rather than
  through any object, so nothing traceable could anchor them. This is the
  pre-existing behaviour for that (AVM1-era) mechanism, made explicit.

### 16.11 Changed files and commits

Corrected fix, on top of `3145f91f3`:

| File | Change |
|---|---|
| `core/src/library.rs` | The lifetime model: `Pin`, weak anchors (`translation_units`), split `trace_anchors` / `trace_contents`, the `Resurrector`, `resurrect_needed` / `drop_unneeded`, `Library::resolve_releasable_libraries`, `set_root_movie`; class-registry cleanup by finalization colour |
| `core/src/player.rs` | `collect_garbage`: mark → finalize (libraries) → resume marking to a fixpoint → sweep; `collect_all_garbage` for tests; external allocation reported every frame |
| `core/src/avm2/script.rs`, `core/src/avm2.rs` | `TranslationUnitWeak`; `do_abc` registers each unit with its movie's library |
| `core/src/avm2/class.rs` | `ClassWeak::is_dead` |
| `core/src/character.rs` | `Character::has_reachable_instances` |
| `core/src/display_object.rs`, `display_object/{movie_clip,graphic,edit_text,text,morph_shape,avm1_button,avm2_button,video,bitmap,loader_display}.rs` | `DisplayObjectWeak::is_dead` and per-type `is_dead` / `shared_data_is_reachable` |
| `core/src/context.rs` | pin the root movie's library |
| `core/src/loader.rs` | pin `ImportAssets` libraries |
| `tests/tests/movie_library/mod.rs`, `tests/tests/regression_tests.rs` | the two new lifetime tests |
| `tests/tests/swfs/avm2/loader_unload_retains_linked_class/` | new: `Test.as`, `child/Child.as`, `build.py`, built SWFs, expected output |

Follow-up commit (section 17):

| File | Change |
|---|---|
| `core/src/avm2/globals/flash/display/loader.rs`, `core/src/context.rs`, `core/src/player.rs` | `unloadAndStop(true)` requests a collection; granted once per ten frames; tessellation eviction pass |
| `core/src/display_object/graphic.rs`, `core/src/tessellation_cache.rs`, `core/src/display_object/movie_clip.rs`, `core/src/library.rs` | tessellation on first draw; parsed shape records read back from the tag on demand; eviction of both after 20 s unused |
| `core/src/bitmap/bitmap_data.rs` | `TrackedPixels`: pixel buffers reported to the collector |
| `core/src/display_object.rs` | `TrackedCacheBitmap`: `cacheAsBitmap`/filter textures reported to the collector |
| `core/src/memory_report.rs`, `desktop/src/memory_reporter.rs`, `desktop/src/main.rs` | GPU counters, mesh and texture accounting, Rust-heap bytes (counting allocator), external bytes, in the CSV and the log line |
| `render/src/backend.rs`, `render/wgpu/Cargo.toml`, `render/wgpu/src/{backend,lib,mesh,buffer_pool}.rs`, `render/wgpu/src/surface/target.rs` | `RenderBackend::memory_usage`; wgpu `counters`; live mesh and texture byte accounting |

Untouched from `463b742fe`: `core/src/avm2/globals/flash/display/Loader.as` and
`loader.rs` (`unloadAndStop`), `core/src/memory_report.rs`, the
`loader_unload_releases_library` test.

| | |
|---|---|
| Corrected fix commit | `8648d3b02d1826b748fcf1df039606146522084a` |
| Report commit | the commit that adds this section |
| Branch | `fix/aqw-memory-leak`, local only; nothing pushed |

## 17. Field report: lag and 3.5 GB after ten map switches

### 17.1 What was reported

After the corrected fix was delivered, the client ran it on Windows and
reported that after ten map switches the process was at **3,569 MB**
(Task Manager) and that the game "starts lagging hard". No memory report was
recorded on that run, and the screenshot does not say which build it was, so
both symptoms were reproduced and diagnosed here.

### 17.2 The lag: a collection storm from `unloadAndStop(true)`

Section 6.4's `unloadAndStop(gc = true)` charged the collector a whole
collection's worth of debt on **every call**. gc-arena carries artificial
debt across cycles and pays *all* outstanding debt in a single
`collect_debt` call, so *n* calls in one frame meant roughly *n* complete
mark-and-sweep cycles inside that frame. AQW calls `unloadAndStop(true)`
once per loader slot it disposes (`LoaderSlot.dispose`, `World.closeLoader`,
`clearLoaders`, `cleanupMap`) — dozens per zone change in a populated room —
and the cost of each cycle grows with the heap. That is a stall that gets
worse the longer the session runs, which is what "after 10 map switches it
starts lagging hard" describes.

Reproduced deterministically (`GcStorm.as`, scratch harness): a 100,000-object
live heap, 40 loaded SWFs, all 40 disposed with `unloadAndStop(true)` in one
frame, frame times measured with `getTimer()`:

| Build | Frame after the dispose | Next 60 frames (should be 2,000 ms at 30 fps) | Worst frame |
|---|---|---|---|
| Unmodified (`unloadAndStop` a stub, no collection) | 33 ms | 2,000 ms | 34 ms |
| Corrected fix as delivered (`8648d3b02`) | 170 ms | **4,391 ms** | **224 ms** |
| With the change below (re-measured on the final build) | 43 ms | **2,000 ms** | 43 ms |

**Change.** `unloadAndStop(true)` now sets a request flag on the update
context instead of charging debt. `Player::update` grants the request with
one full cycle at the end of that update, at most once every 10 frames; any
number of requests in that window cost one cycle, and requests inside the
window are simply left to the normally paced collector, which the libraries'
external-memory accounting already keeps in step with what was dropped. The
limit is counted in frames rather than time so that the test suite behaves
the same however fast it runs. With AQW's heap (about 100 MB, 350,000
objects) a full cycle costs about 50 ms, so the worst case is now one such
cycle every ten frames during a transition, instead of one per disposed
loader per frame.

### 17.3 The memory: what a real ten-switch session holds

The corrected build was run through the same test here — authenticated
session, populated rooms, `/join` between Battleon, Yulgar and the farm ten
times, 30 s apart — with the memory report extended to include the render
backend's own counters (`gpu_textures`, `gpu_texture_bytes`,
`gpu_buffer_bytes`; the byte counters are populated on Vulkan, DX12 and Metal,
and read zero on OpenGL, which is what this machine's Xvfb display provides).

| Point | Movies resident | Characters | Library content (SWF + decoded bitmaps) | GC heap | RSS |
|---|---|---|---|---|---|
| After the 3rd switch (peak RSS) | 225 | 26,153 | 46 MiB | — | **1,489 MiB** |
| After the 10th switch | 319 | 28,648 | 46 MiB | — | 1,279 MiB |
| 3 minutes later, loads settled | 462–484 | 36,678 | 46 MiB | 108–115 MiB | 1,394 MiB |

No `non-registered character`, no `Error #2136`, no panic, and no stall in
any of the ten transitions. The maps themselves are released: none of the
three appears among the largest resident movies afterwards. What stays is
the equipment of every distinct player seen — the server issued 804
`equipItem` commands naming **439 distinct files** in this session, and
AQW's shared `World.loaderD` domain keeps each of them resolvable (section
16.2), so one library per distinct asset is the correct, Flash-faithful
working set.

But the libraries themselves account for only about 50 MiB of the 1.4 GiB,
and the GC heap for about 110 MiB. A memory-map breakdown of the running
process put **1,270 MiB in the Rust heap** and almost nothing in driver or
shared mappings, so the rest is heap memory that scales with the number of
resident assets and is not part of the library accounting.

### 17.4 The rest: what a resident asset costs, and making it cost less

The memory report was extended so that the remaining memory could be
attributed instead of guessed at: the render backend's own counters
(`gpu_textures`, `gpu_texture_bytes`, `gpu_buffer_bytes`; bytes are only
populated on Vulkan, DX12 and Metal), Ruffle's own count of tessellated
meshes and their bytes (`meshes`, `mesh_bytes`), Ruffle's own count of the
textures it has created and not yet released (`tracked_textures`,
`tracked_texture_bytes`, which works on every backend), and the bytes live in
the Rust allocator (`rust_heap_bytes`, from a counting global allocator in
the desktop player). Repeating the ten-switch session with those counters
gave, after the tenth switch:

| Bucket | Corrected fix as delivered | Notes |
|---|---|---|
| Process RSS | ~1,900 MiB | of which ~1,050 MiB is the software GL driver on this machine (textures live in system RAM here) |
| Rust heap | ~790 MiB | everything Ruffle itself allocates |
| – of which GC heap (`gc_allocation`) | ~130 MiB | ActionScript objects |
| – of which library content (SWF data + decoded bitmaps) | ~50 MiB | the movie libraries proper |
| – of which tessellated meshes | ~20 MiB | vertex and index data of the shapes actually drawn |
| – remainder | ~500 MiB | grows with resident movies |
| Tracked textures | ~500–650 MiB | bitmaps, cached display objects, render targets |

Two things account for the "remainder" and for most of the textures, and
both were changed.

**Parsed shape records.** Every `DefineShape` in a resident SWF kept its
parsed form (`swf::Shape`: style tables and a vector of edge records) for as
long as the library lived. A detailed AQW shape is tens of kilobytes parsed
against a few hundred bytes in the SWF, and a resident asset has dozens of
them, so the ~470 assets AQW's shared domain keeps resolvable cost roughly a
megabyte each — an order of magnitude more than the SWF bytes, which are
retained anyway. Tessellated meshes were the same story one level up: every
shape was tessellated into a GPU mesh **at load time** and kept, with up to
four scaled variants, whether or not it was ever drawn.

*Change.* A shape is now tessellated on first draw, at the scale it is drawn
at (the code path that already existed for other scales), and `Player::update`
drops both the meshes and the parsed records of any shape that has not been
drawn or hit-tested for 20 seconds (checked every 2 seconds). The records
are read back from the retained SWF bytes on the next use; the bounds, which
are what layout and culling need, are kept. Nothing about what is drawn
changes — the full regression suite, most of it image comparisons, passes
unchanged — only *when* the expensive representation exists.

**`BitmapData` pixel buffers.** AQW renders every avatar in a room to a
`BitmapData` (its `mcImages`), and disposes them when the avatar leaves. The
collector counts only the small `Gc` box such an object lives in, not the
pixel buffer behind it, so a room's worth of discarded bitmaps — and the GPU
textures that go with them — could sit unreachable for a whole collection
cycle. *Change.* Pixel buffers now report their size to the collector's
pacing (`TrackedPixels` in `bitmap_data.rs`), so allocating them brings the
next cycle forward and freeing them is credited, exactly as the movie
libraries' bytes already were.

**`cacheAsBitmap` and filter textures.** The same blind spot, one level up:
a display object with filters or `cacheAsBitmap` owns a texture the size of
its bounds, held in a small `Gc` box. A room full of avatars and name labels
with filters is several hundred such textures, and when the room is left
they stay until the collector next reaches their display objects. *Change.*
Those textures now report their size to the collector too
(`TrackedCacheBitmap` in `display_object.rs`).

### 17.4.1 Results

The ten-zone-change session was repeated after each change, same rooms,
same account, same `/join` sequence. The rooms got busier as the evening went
on (the server named 447 distinct equipment files in the first run and 488
in the last), so the honest comparison is the shape of each run and the
per-asset cost, not one number.

| Build | RSS after the 10th switch | Rust heap | Resident movies | RSS at end of settle | Meshes | Tracked textures |
|---|---|---|---|---|---|---|
| Corrected fix as delivered (`8648d3b02`) + lag fix | 1,279 MiB | not counted | 319 | 1,394 MiB (462 movies) | — | — |
| + counters (same code) | 1,860 MiB | 788 MiB | 448 | 2,039 MiB (596) | 4,554 / 16 MiB | — |
| + lazy tessellation | 1,575 MiB | — | 357 | 1,570 MiB (354) | — | — |
| + on-demand shape records | 1,306 MiB | 524 MiB | 398 | 1,442 MiB (511) | 4,933 / 19 MiB | 657 / 507 MiB |
| + `BitmapData` pixels reported | 744 MiB | 341 MiB | 210 | 1,278 MiB (588, busiest run) | 3,956 / 14 MiB | 273 / 216 MiB |
| **+ cache textures reported (final)** | **1,041 MiB** | **484 MiB** | 388 | 1,323 MiB (588) | 4,568 / 16 MiB | 596 / 386 MiB |

The two final rows are the busiest rooms of the evening (130 resident
movies before the first switch, against 64–99 earlier); read per resident
asset, process memory went from about 3.4 MiB per asset on the build the
client tested to about 2.2 MiB on the final one, and the Rust heap from
about 1.7 MiB per asset to about 1.25 MiB, on this machine where all
textures live in the software renderer's system memory. On the final build
RSS stopped growing from the fifth switch on (1,100 → 1,080 → 1,081 →
1,081 → 1,081 → 1,041 MiB) while resident movies kept rising, which is the
new representation doing its job: what is on screen costs memory, what the
game merely keeps resolvable costs a fraction of what it did.

The tracked-texture figure on the final build is flat at ~390 MiB across all
ten switches; before the two accounting changes it climbed from 240 to
660 MiB over the same sequence, which was discarded avatar bitmaps waiting
for a collection that the collector saw no reason to run.

No `non-registered character`, no `Error #2136` and no panic in any of the
runs, and the full regression suite passes unchanged (4,216 / 0 / 349) after
each of the changes, so nothing rendered has changed.

**Baseline behaviour, re-checked on the final build** (section 16.8's
method, quiet machine, harness `maps` scenario, 20 genuine map loads):

| Measure | Baseline | Final build |
|---|---|---|
| Backend ready → first harness frame | 0.225 s | 0.256 s |
| Backend ready → first AQW map fully loaded | 0.622 s | 0.526 s |
| CPU time for the 110 s run (user + system) | 12.19 + 3.69 = 15.9 s | 9.45 + 2.26 = 11.7 s |
| Peak RSS of that run | 375 MiB | 197 MiB |
| Loads completed / movies resident at end | 20 / 21 | 20 / 3 |

Tessellating on first draw rather than at load moves work from loading to
the first frame a shape appears in, and this run's numbers do not show it:
the first map is on screen sooner and the run as a whole costs less CPU
than the baseline, which spends its time keeping everything it ever loaded.
Single runs, so "no detected regression" is the claim, not equivalence.

What remains per resident asset — roughly a megabyte on the Rust heap —
is the asset's other parsed definitions (sprite timelines, text, fonts,
buttons, the ActionScript of its `SymbolClass` links) and its share of the
ActionScript heap. Making those on-demand as well is the same idea applied
to every character type, and is the next step if the client's own numbers
(17.5) say the working set is still too large for their machine.


### 17.5 What the client should send

The Windows figure of 3,569 MB could not be reproduced here because the
machines differ in renderer (DX12 versus software OpenGL), and the
screenshot does not identify the build. The memory report now records
enough to settle it remotely: run with

```
ruffle_desktop --tcp-connections allow --base "https://game.aq.com/game/" \
  --memory-report aqw.csv --memory-report-interval 5 \
  "https://game.aq.com/game/gamefiles/Loader3.swf"
```

and send `aqw.csv` together with the console output. The `movies`,
`characters`, `gc_allocation` and `gpu_*` columns say, sample by sample,
whether the growth is in libraries, in the ActionScript heap, or in the
renderer, and the console shows which build is running.
