function printBytes() {
    trace("  loaded: " + _root.getBytesLoaded());
    trace("  total: " + _root.getBytesTotal());
}

trace("Start");
printBytes();

_root.i = 0;
_root.onEnterFrame = function() {
    _root.i += 1;
    trace("Frame " + _root.i);
    printBytes();

    if (_root.i == 5) {
        _root.onEnterFrame = null;
    }
}

trace("Finished");
