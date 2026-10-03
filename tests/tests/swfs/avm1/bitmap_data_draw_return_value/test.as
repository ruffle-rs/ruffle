var map = _root.createEmptyMovieClip("map", 1);

map.beginFill(0xFF0000, 100);
map.moveTo(0, 0);
map.lineTo(10, 0);
map.lineTo(10, 10);
map.lineTo(0, 10);
map.lineTo(0, 0);
map.endFill();

// 1. Valid BitmapData
var src = new flash.display.BitmapData(10, 10, true, 0xFFFF0000);
var dest1 = new flash.display.BitmapData(10, 10, true, 0);

var result1 = dest1.draw(src);

trace("valid BitmapData: " + result1);
trace("valid BitmapData pixel: " + dest1.getPixel32(5, 5).toString(16));

// 2. Valid MovieClip
var dest2 = new flash.display.BitmapData(10, 10, true, 0);

var result2 = dest2.draw(map);

trace("valid MovieClip: " + result2);
trace("valid MovieClip pixel: " + dest2.getPixel32(5, 5).toString(16));

// 3. Disposed BitmapData
var disposed = new flash.display.BitmapData(10, 10, true, 0xFFFF0000);
disposed.dispose();

var dest3 = new flash.display.BitmapData(10, 10, true, 0);
var result3 = dest3.draw(disposed);

trace("disposed BitmapData: " + result3);

// 4. Plain object
var dest4 = new flash.display.BitmapData(10, 10, true, 0);
var result4 = dest4.draw({});

trace("plain object: " + result4);

// 5. Number
var dest5 = new flash.display.BitmapData(10, 10, true, 0);
var result5 = dest5.draw(123);

trace("number: " + result5);

// 6. null
var dest6 = new flash.display.BitmapData(10, 10, true, 0);
var result6 = dest6.draw(null);

trace("null: " + result6);

// 7. undefined
var dest7 = new flash.display.BitmapData(10, 10, true, 0);
var result7 = dest7.draw(undefined);

trace("undefined: " + result7);
