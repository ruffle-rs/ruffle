function test(s1, s1Name) {
  trace("Testing " + s1Name);

  trace("  s1.global:          " + (s1.global));
  trace("  s1.global2:         " + (s1.global2));
  trace("  s1.anObject:        " + (s1.anObject));
  trace("  s1.anArray:         " + (s1.anArray));
  trace("  s1.aFunction:       " + (s1.aFunction));
  trace("  s1.anObjectClass:   " + (s1.anObjectClass));
  trace("  s1.aMovieClipClass: " + (s1.aMovieClipClass));
  trace("  s1.aBooleanClass:   " + (s1.aBooleanClass));

  trace("  typeof s1.global:          " + (typeof s1.global));
  trace("  typeof s1.global2:         " + (typeof s1.global2));
  trace("  typeof s1.anObject:        " + (typeof s1.anObject));
  trace("  typeof s1.anArray:         " + (typeof s1.anArray));
  trace("  typeof s1.aFunction:       " + (typeof s1.aFunction));
  trace("  typeof s1.anObjectClass:   " + (typeof s1.anObjectClass));
  trace("  typeof s1.aMovieClipClass: " + (typeof s1.aMovieClipClass));
  trace("  typeof s1.aBooleanClass:   " + (typeof s1.aBooleanClass));

  trace("  typeof s1.global.prototype:          " + (typeof s1.global.prototype));
  trace("  typeof s1.global2.prototype:         " + (typeof s1.global2.prototype));
  trace("  typeof s1.anObject.prototype:        " + (typeof s1.anObject.prototype));
  trace("  typeof s1.anArray.prototype:         " + (typeof s1.anArray.prototype));
  trace("  typeof s1.aFunction.prototype:       " + (typeof s1.aFunction.prototype));
  trace("  typeof s1.anObjectClass.prototype:   " + (typeof s1.anObjectClass.prototype));
  trace("  typeof s1.aMovieClipClass.prototype: " + (typeof s1.aMovieClipClass.prototype));
  trace("  typeof s1.aBooleanClass.prototype:   " + (typeof s1.aBooleanClass.prototype));

  trace("  true instanceof s1.aBooleanClass:    " + (true instanceof s1.aBooleanClass));
  trace("  _root instanceof s1.aMovieClipClass: " + (_root instanceof s1.aMovieClipClass));
  trace("  {} instanceof s1.anObjectClass:      " + ({} instanceof s1.anObjectClass));
}

function compare(s1, s2, s1Name, s2Name) {
  trace("Comparing " + s1Name + " and " + s2Name);

  trace("  s1.global          == s2.global:          " + (s1.global          == s2.global));
  trace("  s1.global2         == s2.global2:         " + (s1.global2         == s2.global2));
  trace("  s1.anObject        == s2.anObject:        " + (s1.anObject        == s2.anObject));
  trace("  s1.anArray         == s2.anArray:         " + (s1.anArray         == s2.anArray));
  trace("  s1.aFunction       == s2.aFunction:       " + (s1.aFunction       == s2.aFunction));
  trace("  s1.anObjectClass   == s2.anObjectClass:   " + (s1.anObjectClass   == s2.anObjectClass));
  trace("  s1.aMovieClipClass == s2.aMovieClipClass: " + (s1.aMovieClipClass == s2.aMovieClipClass));
  trace("  s1.aBooleanClass   == s2.aBooleanClass:   " + (s1.aBooleanClass   == s2.aBooleanClass));

  trace("  s1.global          === s2.global:          " + (s1.global          === s2.global));
  trace("  s1.global2         === s2.global2:         " + (s1.global2         === s2.global2));
  trace("  s1.anObject        === s2.anObject:        " + (s1.anObject        === s2.anObject));
  trace("  s1.anArray         === s2.anArray:         " + (s1.anArray         === s2.anArray));
  trace("  s1.aFunction       === s2.aFunction:       " + (s1.aFunction       === s2.aFunction));
  trace("  s1.anObjectClass   === s2.anObjectClass:   " + (s1.anObjectClass   === s2.anObjectClass));
  trace("  s1.aMovieClipClass === s2.aMovieClipClass: " + (s1.aMovieClipClass === s2.aMovieClipClass));
  trace("  s1.aBooleanClass   === s2.aBooleanClass:   " + (s1.aBooleanClass   === s2.aBooleanClass));

  trace("  s1.global.prototype          == s2.global.prototype:          " + (s1.global.prototype          == s2.global.prototype));
  trace("  s1.global2.prototype         == s2.global2.prototype:         " + (s1.global2.prototype         == s2.global2.prototype));
  trace("  s1.anObject.prototype        == s2.anObject.prototype:        " + (s1.anObject.prototype        == s2.anObject.prototype));
  trace("  s1.anArray.prototype         == s2.anArray.prototype:         " + (s1.anArray.prototype         == s2.anArray.prototype));
  trace("  s1.aFunction.prototype       == s2.aFunction.prototype:       " + (s1.aFunction.prototype       == s2.aFunction.prototype));
  trace("  s1.anObjectClass.prototype   == s2.anObjectClass.prototype:   " + (s1.anObjectClass.prototype   == s2.anObjectClass.prototype));
  trace("  s1.aMovieClipClass.prototype == s2.aMovieClipClass.prototype: " + (s1.aMovieClipClass.prototype == s2.aMovieClipClass.prototype));
  trace("  s1.aBooleanClass.prototype   == s2.aBooleanClass.prototype:   " + (s1.aBooleanClass.prototype   == s2.aBooleanClass.prototype));

  trace("  s1.global.prototype          === s2.global.prototype:          " + (s1.global.prototype          === s2.global.prototype));
  trace("  s1.global2.prototype         === s2.global2.prototype:         " + (s1.global2.prototype         === s2.global2.prototype));
  trace("  s1.anObject.prototype        === s2.anObject.prototype:        " + (s1.anObject.prototype        === s2.anObject.prototype));
  trace("  s1.anArray.prototype         === s2.anArray.prototype:         " + (s1.anArray.prototype         === s2.anArray.prototype));
  trace("  s1.aFunction.prototype       === s2.aFunction.prototype:       " + (s1.aFunction.prototype       === s2.aFunction.prototype));
  trace("  s1.anObjectClass.prototype   === s2.anObjectClass.prototype:   " + (s1.anObjectClass.prototype   === s2.anObjectClass.prototype));
  trace("  s1.aMovieClipClass.prototype === s2.aMovieClipClass.prototype: " + (s1.aMovieClipClass.prototype === s2.aMovieClipClass.prototype));
  trace("  s1.aBooleanClass.prototype   === s2.aBooleanClass.prototype:   " + (s1.aBooleanClass.prototype   === s2.aBooleanClass.prototype));

  trace("  s1.global.__proto__          == s2.global.__proto__:          " + (s1.global.__proto__          == s2.global.__proto__));
  trace("  s1.global2.__proto__         == s2.global2.__proto__:         " + (s1.global2.__proto__         == s2.global2.__proto__));
  trace("  s1.anObject.__proto__        == s2.anObject.__proto__:        " + (s1.anObject.__proto__        == s2.anObject.__proto__));
  trace("  s1.anArray.__proto__         == s2.anArray.__proto__:         " + (s1.anArray.__proto__         == s2.anArray.__proto__));
  trace("  s1.aFunction.__proto__       == s2.aFunction.__proto__:       " + (s1.aFunction.__proto__       == s2.aFunction.__proto__));
  trace("  s1.anObjectClass.__proto__   == s2.anObjectClass.__proto__:   " + (s1.anObjectClass.__proto__   == s2.anObjectClass.__proto__));
  trace("  s1.aMovieClipClass.__proto__ == s2.aMovieClipClass.__proto__: " + (s1.aMovieClipClass.__proto__ == s2.aMovieClipClass.__proto__));
  trace("  s1.aBooleanClass.__proto__   == s2.aBooleanClass.__proto__:   " + (s1.aBooleanClass.__proto__   == s2.aBooleanClass.__proto__));

  trace("  s1.global.__proto__          === s2.global.__proto__:          " + (s1.global.__proto__          === s2.global.__proto__));
  trace("  s1.global2.__proto__         === s2.global2.__proto__:         " + (s1.global2.__proto__         === s2.global2.__proto__));
  trace("  s1.anObject.__proto__        === s2.anObject.__proto__:        " + (s1.anObject.__proto__        === s2.anObject.__proto__));
  trace("  s1.anArray.__proto__         === s2.anArray.__proto__:         " + (s1.anArray.__proto__         === s2.anArray.__proto__));
  trace("  s1.aFunction.__proto__       === s2.aFunction.__proto__:       " + (s1.aFunction.__proto__       === s2.aFunction.__proto__));
  trace("  s1.anObjectClass.__proto__   === s2.anObjectClass.__proto__:   " + (s1.anObjectClass.__proto__   === s2.anObjectClass.__proto__));
  trace("  s1.aMovieClipClass.__proto__ === s2.aMovieClipClass.__proto__: " + (s1.aMovieClipClass.__proto__ === s2.aMovieClipClass.__proto__));
  trace("  s1.aBooleanClass.__proto__   === s2.aBooleanClass.__proto__:   " + (s1.aBooleanClass.__proto__   === s2.aBooleanClass.__proto__));

  trace("  s1.anObject instanceof s2.anObjectClass: " + (s1.anObject instanceof s2.anObjectClass));
}


var mc5 = createEmptyMovieClip("child5", 2);
var mc6 = createEmptyMovieClip("child6", 3);
var mc7 = createEmptyMovieClip("child7", 4);
var mc8 = createEmptyMovieClip("child8", 5);
var mc9 = createEmptyMovieClip("child9", 6);

var loader5 = new MovieClipLoader();
loader5.loadClip("child5.swf", mc5);

var loader6 = new MovieClipLoader();
loader6.loadClip("child6.swf", mc6);

var loader7 = new MovieClipLoader();
loader7.loadClip("child7.swf", mc7);

var loader8 = new MovieClipLoader();
loader8.loadClip("child8.swf", mc8);

var loader9 = new MovieClipLoader();
loader9.loadClip("child9.swf", mc9);

var toLoad = 5;
var listener = {
  onLoadInit: function() {
    if (--toLoad == 0) {
      test(mc5, "SWF5");
      test(mc6, "SWF6");
      test(mc7, "SWF7");
      test(mc8, "SWF8");
      test(mc9, "SWF9");
      compare(mc5, mc6, "SWF5", "SWF6");
      compare(mc5, mc7, "SWF5", "SWF7");
      compare(mc5, mc8, "SWF5", "SWF8");
      compare(mc5, mc9, "SWF5", "SWF9");
      compare(mc6, mc5, "SWF6", "SWF5");
      compare(mc6, mc7, "SWF6", "SWF7");
      compare(mc6, mc8, "SWF6", "SWF8");
      compare(mc6, mc9, "SWF6", "SWF9");
      compare(mc7, mc5, "SWF7", "SWF5");
      compare(mc7, mc6, "SWF7", "SWF6");
      compare(mc7, mc8, "SWF7", "SWF8");
      compare(mc7, mc9, "SWF7", "SWF9");
      compare(mc8, mc5, "SWF8", "SWF5");
      compare(mc8, mc6, "SWF8", "SWF6");
      compare(mc8, mc7, "SWF8", "SWF7");
      compare(mc8, mc9, "SWF8", "SWF9");
      compare(mc9, mc5, "SWF9", "SWF5");
      compare(mc9, mc6, "SWF9", "SWF6");
      compare(mc9, mc7, "SWF9", "SWF7");
      compare(mc9, mc8, "SWF9", "SWF8");
    }
  }
};

loader5.addListener(listener);
loader6.addListener(listener);
loader7.addListener(listener);
loader8.addListener(listener);
loader9.addListener(listener);
