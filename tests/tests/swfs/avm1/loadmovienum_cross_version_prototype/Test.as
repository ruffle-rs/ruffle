// Compile with:
//  mtasc -main -header 200:150:30 -version 6 Test.as -swf test.swf

class Test {
  static var checkCount = 0;
  static var phase = 0;

  static function main(current) {
    trace("Test (SWF6): Loading target.swf into _level2");
    getURL("target.swf", "_level2");

    current.onEnterFrame = function() {
      Test.checkCount++;

      if (Test.phase == 0) {
        // Phase 0: Wait for load and test prototype
        if (_level2 != undefined && _level2._url != undefined && _level2._url.indexOf("target.swf") >= 0) {
          trace("Test (SWF6): _level2.getSWFVersion() = " + _level2.getSWFVersion());

          MovieClip.prototype.testProp = "FROM_SWF6";
          trace("Test (SWF6): _level2.testProp = " + _level2.testProp);
          trace("Test (SWF6): _level2.__proto__ === MovieClip.prototype: " + (_level2.__proto__ === MovieClip.prototype));

          if (_level2.runProtoTest != undefined) {
            _level2.runProtoTest();
          }

          // Now test unload behavior
          trace("Test (SWF6): Unloading _level2");
          getURL("", "_level2");
          Test.phase = 1;
          Test.checkCount = 0;
        } else if (Test.checkCount > 60) {
          trace("Test (SWF6): Timeout");
          delete current.onEnterFrame;
        }
      } else if (Test.phase == 1) {
        // Phase 1: Check state after unload (wait a couple frames)
        if (Test.checkCount >= 2) {
          trace("Test (SWF6): After unload - _level2.__proto__ === MovieClip.prototype: " + (_level2.__proto__ === MovieClip.prototype));
          // The prototype should still be SWF8's, not SWF6's, because the unloaded
          // stub keeps the loaded movie's version
          trace("Test (SWF6): After unload - _level2.testProp = " + _level2.testProp);
          delete current.onEnterFrame;
        }
      }
    };
  }
}
