// Compile with:
//  mtasc -main -header 200:150:30 -version 8 Target.as -swf target.swf

class Target {
  static function main(current) {
    current.runProtoTest = function() {
      MovieClip.prototype.customProp = "FROM_SWF8";
      trace("Target (SWF8): _level2.customProp = " + _level2.customProp);
      trace("Target (SWF8): _level2.__proto__ === MovieClip.prototype: " + (_level2.__proto__ === MovieClip.prototype));
    };
  }
}
