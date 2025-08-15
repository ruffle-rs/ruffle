package {
import flash.display.MovieClip;
import flash.events.Event;

public class Test extends MovieClip {
    public function Test() {
        var that:Test = this;
        stage.addEventListener(Event.ENTER_FRAME, function (event:Event):void {
            var currentFrame = [that.currentScene.name, that.currentFrame];
            trace("Entered frame: " + currentFrame);
        });

        this.gotoAndStop(2, "scene");
    }
}
}
