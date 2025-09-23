package {
import flash.display.*;
import flash.utils.*;
import flash.events.*;

public class Test extends MovieClip {
    public function Test() {
        var t:Timer = new Timer(100, 2);
        t.addEventListener("timer", timer);
        t.addEventListener("timerComplete", timerComplete);
        t.start();
    }

    public function timer(event:TimerEvent):void {
        trace("timer: " + event);
    }

    public function timerComplete(event:TimerEvent):void {
        trace("timerComplete: " + event);
    }
}
}
