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

**What changed.** Four source-level changes in `ruffle_core`, described in
section 6: a weakly-held content-root handle that lets a loaded movie's library
be dropped once its content is unreachable; weak keys in the AVM2 class
registry; external-memory accounting so the collector paces itself against the
memory actually in play; and a real implementation of `Loader.unloadAndStop`,
which AQW calls on every zone change and which was previously a stub.

**Is it fixed.** The unbounded retention is fixed under the agreed reproduction
(sections 8 and 9). **However, authenticated live AQW testing subsequently found
a correctness defect in the fix** — it releases a movie's library while
ActionScript still holds a class from that movie, which is exactly AQW's
equipment pattern. **This fix must not ship in its current form.** See section 15.


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

## 6. The fix

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

## 10. Regression verification

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
cargo test --package tests --test tests -- loader_unload_releases_library
cargo fmt --all -- --check
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
| Fix commit | `463b742fe2e1945f013313d98344843e707c3c4e` |
| Branch | `fix/aqw-memory-leak` (local only; nothing was pushed) |

The instrumentation is a separate first commit specifically so the baseline
binary used for every "before" number in this report could be built from it and
differs from the patched binary in nothing but the fix.

## 14. Limitations

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

