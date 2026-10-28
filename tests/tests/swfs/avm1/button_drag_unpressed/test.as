var events = ["onRollOver", "onRollOut", "onDragOver", "onDragOut", "onPress", "onRelease", "onReleaseOutside"];
var loader = new MovieClipLoader();
var loaded = {};
loaded.onLoadInit = function (clip) {
    for (var i = 0; i < 3; i++) {
        var tile = clip.attachMovie("tile", "tile" + i, i + 1);
        tile._x = 20 + i * 100;
        tile._y = 20;
        for (var j = 0; j < events.length; j++) {
            tile[events[j]] = function () {
                var message = this._name + " " + arguments.callee.eventName;
                trace(message);
            };
            tile[events[j]].eventName = events[j];
        }
    }
};
loader.addListener(loaded);
loader.loadClip("buttons.swf", _root.createEmptyMovieClip("buttons", 1));
stop();

// Distinct artwork widths let the trace verify the native button state.
var listener = {};
listener.onKeyDown = function () {
    if (Key.getCode() == 77) {
        _root.buttons.tile1.trackAsMenu = true;
        return;
    }
    var message = _root.buttons.tile0._width + "," + _root.buttons.tile1._width + "," + _root.buttons.tile2._width;
    trace(message);
};
Key.addListener(listener);
