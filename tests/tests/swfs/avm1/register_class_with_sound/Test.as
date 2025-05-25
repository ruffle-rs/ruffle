// Compile with:
//  mtasc -main -version 8 Test.as -swf assets.swf -out test.swf

// Regression test for https://github.com/ruffle-rs/ruffle/issues/20475
class Test extends Sound {

  function Test() {
    super();
    trace("typeof this = " + (typeof this));
    trace("this instanceof Sound = " + (this instanceof Sound));
    trace("this instanceof MovieClip = " + (this instanceof MovieClip));
    this["_x"] = 5.52;
    trace("this._name = " + this["_name"]);
    trace("this._x = " + this["_x"]);

    trace("this.getVolume() = " + this.getVolume());
    this.setVolume(10);
    trace("this.getVolume() = " + this.getVolume());
    trace("this.getTransform() = " + this.getTransform());
  }

  static function main(current) {
    Object.registerClass("sprite", Test);

    // This seems to produce a fully-functional `Sound` object in FP,
    // but not in Ruffle.
    trace("Construct through attachMovie");
    current.attachMovie("sprite", "instance", 1);
    trace("");

    // Calling Sound methods on a 'dummy' object doesn't work at all.
    trace("getVolume.call({}) = " + Sound.prototype.getVolume.call({}));
  }
}
