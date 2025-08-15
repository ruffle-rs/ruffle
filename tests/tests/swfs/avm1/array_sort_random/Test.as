// Compile with:
//  mtasc -main -version 8 Test.as -out test.swf 
class Test {

  static function main(current) {
    var array = [];
    for (var i = 0; i < 50; i++) {
      array.push(i);
    }

    // "sort" the array using randomly-chosen comparison results.
    array.sort(function(a, b) {
      var r = Test.rng();
      if (r % 8 == 0) {
        trace("cmp: " + a + " == " + b);
        return 0;
      } else if (r > 0) {
        trace("cmp: " + a + " > " + b);
        return 1;
      } else {
        trace("cmp: " + a + " < " + b);
        return -1;
      }
    });

    trace("// contents of array");
    trace(array);
  }

  // A simple deterministic PRNG; namely, Xorshift.
  static var rngState = 0x12345678;
  static function rng() {
    rngState ^= rngState << 13;
    rngState ^= rngState >>> 17;
    rngState ^= rngState << 5;
    return rngState;
  }
}
