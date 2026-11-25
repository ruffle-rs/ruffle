function getMenu() {
    this.reads = this.reads + 1;
    trace(this._name + " get " + this.menu);
    return this.menu;
}

var tiles = [];
var menu = false;
var loader = new MovieClipLoader();
var loaded = {};
loaded.onLoadInit = function (clip) {
    for (var row = 0; row < 2; row++) {
        for (var i = 0; i < 3; i++) {
            var tile = clip.attachMovie(row == 0 ? "tile" : "movie_tile", (row == 0 ? "button" : "clip") + i, row * 3 + i + 1);
            tile._x = 20 + i * 100;
            tile._y = 20 + row * 120;
            tile.menu = false;
            tile.reads = 0;
            tile.rollOvers = 0;
            tile.rollOuts = 0;
            tile.addProperty("trackAsMenu", getMenu, null);
            tile.onRollOver = function () { this.rollOvers = this.rollOvers + 1; };
            tile.onRollOut = function () { this.rollOuts = this.rollOuts + 1; };
            tile.onPress = function () { trace(this._name + " press"); };
            tile.onDragOver = function () { trace(this._name + " dragOver"); };
            tile.onDragOut = function () { trace(this._name + " dragOut"); };
            tile.onRelease = function () { trace(this._name + " release"); };
            tile.onReleaseOutside = function () { trace(this._name + " releaseOutside"); };
            tiles.push(tile);
        }
    }
};
loader.addListener(loaded);
loader.loadClip("buttons.swf", _root.createEmptyMovieClip("buttons", 1));
stop();

var keys = {};
keys.onKeyDown = function () {
    if (Key.getCode() == 82) {
        trace("reset");
        for (var i = 0; i < tiles.length; i++) {
            tiles[i].reads = 0;
            tiles[i].rollOvers = 0;
            tiles[i].rollOuts = 0;
        }
    }
    if (Key.getCode() == 77) {
        menu = !menu;
        trace("menu " + menu);
        for (var i = 0; i < tiles.length; i++) {
            tiles[i].menu = menu;
            tiles[i].reads = 0;
        }
    }
    if (Key.getCode() == 32) {
        for (var row = 0; row < 2; row++) {
            var values = [];
            for (var i = 0; i < 3; i++) {
                var tile = tiles[row * 3 + i];
                values.push(tile._width + "/" + tile.reads + "/" + tile.rollOvers + "/" + tile.rollOuts);
            }
            trace((row == 0 ? "buttons " : "clips ") + values.join(","));
        }
    }
};
Key.addListener(keys);
