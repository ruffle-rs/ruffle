// Compile with:
//  mtasc -main -header 200:150:30 -version 8 Test.as -swf test.swf 

class Test extends flash.filters.BlurFilter {
  // Uses `BlurFilter` as it is a very simple native object,
  // and so is unlikely to have any special behaviors.

  static function main(current) {
    var t = new Test();
  }

  function Test() {
    this.printFields();

    trace("// super(10, 20, 4)");
    super(10, 20, 4);
    this.printFields();

    var noisyFive = {
      valueOf: function() {
        trace("valueOf called!");
        return 5;
      }
    };

    // This shouldn't change the object.
    trace("// super(20, 10, noisyFive)");
    super(20, 10, noisyFive);
    this.printFields();
  }

  function printFields() {
    trace("this.blurX: " + this.blurX);
    trace("this.blurY: " + this.blurY);
    trace("this.quality: " + this.quality);
  }
}
