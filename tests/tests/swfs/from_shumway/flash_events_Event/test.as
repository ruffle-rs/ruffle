/*
 Compiled with:
 node utils/compileabc.js --swf EventTest,100,100,60 -p test/swfs/flash_events_Event.as
 */

﻿
package {

import flash.display.MovieClip;
import flash.display.Sprite;
import flash.events.Event;
import flash.system.fscommand;

public class EventTest extends MovieClip {
  var s0:Sprite;
  var s1:Sprite;

  public function EventTest() {
    addFrameScript.call(this, 0, frame1);

    s0 = new Sprite();
    s0.addEventListener('test', function (e:TestEvent) {
      trace('s0 event: ' + e.type + ' ' + e.test);
      s1.dispatchEvent(e);
    });
    addChild(s0);
    s1 = new Sprite();
    s1.addEventListener('test', function (e:TestEvent) {
      trace('s1 event: ' + e.type + ' ' + e.test);
    });
    addChild(s1);
  }

  private function frame1() {
    trace('dispatch');
    var e:TestEvent = new TestEvent('test', true, true, 4);
    s0.dispatchEvent(e);

    fscommand('quit');
  }
}

class TestEvent extends Event {
  public var test:Number;

  public function TestEvent(type:String, bubbles:Boolean, cancelable:Boolean, test_:Number) {
    super(type, bubbles, cancelable);
    test = test_;
  }

  public override function clone():Event {
    return new TestEvent(type, bubbles, cancelable, test);
  }
}
}
