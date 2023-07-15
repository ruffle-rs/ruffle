// Compile with:
//  mtasc -main -header 200:200:30 -version 8  Test.as -swf test.swf 
class Test {
  var root;
  var view;
  var viewCache;

  static function main(current) {
    var test = new Test(current);

    // Display a red square...
    test.attach(true, 0xFF0000);

    // ...and remove it.
    test.detach();

    // Display a green square with `_visible = false`:
    // - on Flash, nothing changes;
    // - on Ruffle, this makes the red square reappear.
    test.attach(false, 0x00FF00);
  }

  function Test(current) {
    this.root = current;
    this.view = null;
    this.viewCache = null;
  }

  function attach(visible, color) {
    this.view = this.root.createEmptyMovieClip("view", 1000);
    var inner = this.view.createEmptyMovieClip("inner", 2000);
    inner._visible = visible;

    inner.beginFill(color);
    inner.moveTo(50, 50);
    inner.lineTo(100, 50);
    inner.lineTo(100, 100);
    inner.lineTo(50, 100);
    inner.endFill();

    this.viewCache.dispose();
    this.viewCache = new flash.display.BitmapData(420, 520, true, 0xFF0000);
    this.viewCache.draw(this.view);
    inner.removeMovieClip();
    this.view.attachBitmap(this.viewCache, 0);
  }

  function detach() {
    this.viewCache.dispose();
    this.view.removeMovieClip();
  }
}
