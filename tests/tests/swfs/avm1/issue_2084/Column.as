class Column extends MovieClip
{
    function attachIcon(n : Number, initObject : Object) {
      var name = "icon_" + n;
      attachMovie("icon", name, getNextHighestDepth(), initObject);
      trace(name + " " + this[name]._width + "," + this[name]._height);
    }

    function onLoad() {
      trace("Column onLoad " + _width + "," + _height);

      // This should appear as a column of images, from top to bottom:
      // 1. square: dimensions are not set
      attachIcon(1, {_y: 20});

      // The remaining icons do not appear due to https://github.com/ruffle-rs/ruffle/issues/3414

      // 2. square: redundant image dimensions
      attachIcon(2, {_y: 40, _width: 20, _height: 20});
      // 3. wide: scaled by _width
      attachIcon(3, {_y: 60, _width: 40});
      // 4. tall and thin: scaled by both _width and _height
      attachIcon(4, {_y: 80, _width: 10, _height: 40});
    }
}
