Regression test for a loaded SWF's library outliving the content it was loaded
for.

`test.swf` loads `child/child.swf` five times over, unloading each one before
loading the next. The trace output only checks that the cycles ran; the
interesting assertion is in the `loader_unload_releases_library` test in
`tests/tests/movie_library.rs`, which checks that the player is no longer
holding a library for any of those children afterwards.

Both SWFs are built from the `.as` sources next to them with `asc.jar`
(`tools/asc/asc.jar`) plus a minimal AVM2 SWF container, rather than from a
`.fla`.
