var loader = new MovieClipLoader();

// #12820: loading into a non-existent _levelN string target must create the
// level and succeed (previously it silently failed and returned false).
trace("loadClip _level10 -> " + loader.loadClip("child.swf", "_level10"));
trace("_level10 created -> " + (_level10 != undefined));

// `_flash` is a legacy synonym of `_level` and must work wherever `_level` does.
trace("loadClip _flash12 -> " + loader.loadClip("child.swf", "_flash12"));
trace("_flash12 === _level12 -> " + (_flash12 === _level12));

// getURL into a `_flash` level (action_get_url path).
getURL("child.swf", "_flash14");
trace("_level14 created -> " + (_level14 != undefined));

// loadMovie into a `_flash` level (action_get_url_2 path).
loadMovie("child.swf", "_flash16");
trace("_level16 created -> " + (_level16 != undefined));

// A non-level string target still resolves as a path, not a level.
trace("loadClip _level0.missing -> " + loader.loadClip("child.swf", "_level0.missing"));
