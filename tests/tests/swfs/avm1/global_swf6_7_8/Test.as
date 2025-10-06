// Compile with:
//  mtasc -main -header 200:150:30 -version 8 Test.as -swf test.swf 

class Test {
  static function main(current) {
    var mc6 = current.createEmptyMovieClip("child6", 1);
    var loader6 = new MovieClipLoader();
    loader6.loadClip("child6.swf", mc6);

    var mc7 = current.createEmptyMovieClip("child7", 2);
    var loader7 = new MovieClipLoader();
    loader7.loadClip("child7.swf", mc7);

    var nbToLoad = 2;
    var listener = {
      onLoadInit: function() {
        if (--nbToLoad == 0) {
          Test.test(mc6, mc7);
        }
      }
    };

    loader6.addListener(listener);
    loader7.addListener(listener);
  }

  static function test(swf6, swf7) {
    var g6 = swf6.global;
    var g7 = swf7.global;
    var g8 = _global;

    trace("typeof g6: " + (typeof g6));
    trace("typeof g7: " + (typeof g7));
    trace("typeof g8: " + (typeof g8));

    // v7 and v8 SWFs have the same _global object
    trace("g7 === g8: " + (g7 === g8));

    // v6 and v7 SWFs have completely separate _global objects
    trace("g6 === g7: " + (g6 === g7));
    trace("typeof g6.Object.prototype: " + (typeof g6.Object.prototype));
    trace("typeof g7.Object.prototype: " + (typeof g7.Object.prototype));
    trace("g6.Object.prototype === g7.Object.prototype: " + (g6.Object.prototype === g7.Object.prototype));

    // ...but case-sensitivity doesn't carry over.
    trace("g6.OBJECT: " + g6.OBJECT);

    // Objects created in a v6 SWF uses the classes from their _global
    var anObject = {}, anArray = [];
    trace("swf6.anObject: " + swf6.anObject);
    trace("swf6.anObject instanceof g6.Object: " + (swf6.anObject instanceof g6.Object));
    trace("swf6.anObject instanceof g7.Object: " + (swf6.anObject instanceof g7.Object));
    trace("swf6.anArray: " + swf6.anArray);
    trace("swf6.anArray instanceof g6.Array: " + (swf6.anArray instanceof g6.Array));
    trace("swf6.anArray instanceof g7.Array: " + (swf6.anArray instanceof g7.Array));

    fscommand("quit");
  }
}
