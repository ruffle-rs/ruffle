class Test {
  static function main() {
    _root.message = "Original Message";

    var loader = new MovieClipLoader();

    var o = new Object();
    
    o.onLoadStart = function(target) {
      target.message = "message from onLoadStart"
    }

    loader.addListener(o);
    loader.loadClip("child.swf", "_level0");
  }
}
