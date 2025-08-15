class Main extends MovieClip {

    var first : Boolean;
    var col_mc_1: Column;
    var col_mc_2: Column;

    function onLoad() {
        trace("onLoad");
        first = true;

        // This column should be visible, see Column for scaling.
        attachMovie("column", "col_mc_1", getNextHighestDepth(), {_x:10});
        // Both _width and _height should be 0.
        trace("col_mc_1 " + col_mc_1._width + "," + col_mc_1._height);

        // This column should not be visible due to passing _width here.
        attachMovie("column", "col_mc_2", getNextHighestDepth(), {_x:100, _width: 20});
        // Both _width and _height should be 0.
        trace("col_mc_2 " + col_mc_2._width + "," + col_mc_2._height);
    }

    function onEnterFrame() {
        if (first) {
          first = false;
          trace("onEnterFrame");
          trace("col1 " + col_mc_1._width + "," + col_mc_1._height);
          trace("col2 " + col_mc_2._width + "," + col_mc_2._height);
        }
    }
}
