package {
import flash.display.MovieClip;
import flash.events.Event;
import flash.utils.getDefinitionByName;

public class Loadee extends MovieClip {
    // test.swf's Log, not a copy of its own.
    private static const Log:Object = getDefinitionByName("Log");

    private var n:String = "loadee " + Log.loadeeName;

    public function Loadee() {
        Log.t(n + " constructor: parent=" + parent + " stage=" + stage + " currentFrame=" + currentFrame);
        addFrameScript(0, frame1, 1, frame2);
        addEventListener(Event.ADDED, function(e:Event):void {
            if (e.target == e.currentTarget) Log.t(n + " added: parent=" + parent);
        });
        addEventListener(Event.ADDED_TO_STAGE, function(e:Event):void { Log.t(n + " addedToStage"); });
        addEventListener(Event.ENTER_FRAME, function(e:Event):void { Log.t(n + " enterFrame: currentFrame=" + currentFrame); });
    }

    private function frame1():void { Log.t(n + " frame 1 script"); }
    private function frame2():void { Log.t(n + " frame 2 script"); stop(); }
}
}
