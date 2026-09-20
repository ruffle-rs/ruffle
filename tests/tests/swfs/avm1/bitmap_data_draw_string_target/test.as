var map = _root.createEmptyMovieClip("map", 1);

map.beginFill(0xFF0000, 100);
map.moveTo(0, 0);
map.lineTo(10, 0);
map.lineTo(10, 10);
map.lineTo(0, 10);
map.lineTo(0, 0);
map.endFill();

var dest = new flash.display.BitmapData(10, 10, true, 0x00000000);

trace("map exists: " + (_root.map != undefined));

var result = dest.draw("map");

if (result == 12345) {
    trace("unexpected draw result");
}

trace("pixel 5,5: " + dest.getPixel32(5, 5).toString(16));
