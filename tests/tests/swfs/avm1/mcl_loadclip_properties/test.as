class Test {
  static function main() {
    var loader = new MovieClipLoader();
    var mc = _root.createEmptyMovieClip("child.swf", 1);
    mc.message = "Original Message";

    var o = new Object();

    o.onLoadInit = function(target) {
      trace("onLoadInit");
      trace(target.message);
      target.message = "message from onLoadInit"
    }
    
    o.onLoadStart = function(target) {
      trace("onLoadStart");
      trace(target.message);
      target.message = "message from onLoadStart"
    }

    o.onLoadComplete = function(target) {
      trace("onLoadComplete");
      trace(target.message);
      target.message = "message from onLoadComplete"
    }

    loader.addListener(o);
    loader.loadClip("child.swf", mc);
  }
}