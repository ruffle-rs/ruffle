class Test {
  static function main() {
    _root.message = "Original Message";

    var loader = new MovieClipLoader();

    var o = new Object();
    o.onLoadStart = function(target) {
      trace(target.message); // load
      _root.message = "Target Message";
      trace(target.message);
    };

    loader.addListener(o);
    loader.loadClip("child.swf", "_level0");
  }
}