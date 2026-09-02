Regression test for a loaded SWF's library being released while a class from
that SWF is still in use.

`test.swf` loads `child/child.swf` into a child `ApplicationDomain`, takes the
`Child` class out of that domain, then unloads the `Loader` and drops every
reference to the loaded content. Later it instantiates the held class. `Child`
is linked by `SymbolClass` to a sprite containing a 100x100 shape, so the
instantiation has to find both characters in the child movie's library:

* `instantiated children=1 width=100` means the library was still there;
* `Error #2136` means the player had released it out from under the class.

Finally the class and domain are released. The Rust tests
`retained_class_keeps_library` and `released_class_frees_library` in
`tests/tests/movie_library/mod.rs` run this movie and check the library's
lifetime from the outside at both points: still resident while the class is
held, and gone once it is released.

Both SWFs are built from the `.as` sources next to them by `build.py`, using
`asc.jar` (`tools/asc/asc.jar`) and a hand-assembled SWF container.
