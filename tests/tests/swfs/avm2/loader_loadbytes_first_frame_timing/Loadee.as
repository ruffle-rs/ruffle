package {
import flash.display.MovieClip;
import flash.events.Event;
import flash.utils.getDefinitionByName;

public class Loadee extends MovieClip {
    // test.swf's Log, not a copy of its own.
    private static const Log:Object = getDefinitionByName("Log");

    private var n:String = "loadee " + Log.loadeeNames.shift();
    private var enteredFrame:Boolean = false;

    public function Loadee() {
        Log.t(n + " constructor: parent=" + parent + " currentFrame=" + currentFrame);
        addFrameScript(0, function():void { Log.t(n + " frame 1 script"); stop(); });
        addEventListener(Event.ADDED_TO_STAGE, function(e:Event):void { Log.t(n + " addedToStage"); });
        addEventListener(Event.ENTER_FRAME, function(e:Event):void {
            if (!enteredFrame) Log.t(n + " first enterFrame");
            enteredFrame = true;
        });
    }
}
}
