package {

import flash.display.MovieClip;
import flash.events.Event;
import flash.events.KeyboardEvent;

public class Test extends MovieClip {
    private var testStage: int = 0;
    private var clip1: MovieClip;
    private var clip2: MovieClip;
    private var clip3: MovieClip;

    public function Test() {
        super();

        clip1 = new MovieClip();
        clip1.tabEnabled = true;
        clip1.tabIndex = 1;
        clip2 = new MovieClip();
        clip2.tabEnabled = true;
        clip2.tabIndex = 2;
        clip3 = new MovieClip();
        clip3.tabEnabled = true;
        clip3.tabIndex = 3;

        stage.addChild(clip1);
        MovieClip(root).addChild(clip2);
        stage.tabChildren = true;
        printProps();

        clip1.addEventListener("focusIn", function (evt:Event):void {
            trace("clip1 focusIn: " + evt.toString());
        });
        clip2.addEventListener("focusIn", function (evt:Event):void {
            trace("clip2 focusIn: " + evt.toString());
        });

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 27) {
                trace("Escape pressed");
                testStage += 1;
                if (testStage == 1) {
                    trace("Setting tabChildren to false");
                    stage.focus = null;
                    stage.tabChildren = false;
                    printProps();
                } else if (testStage == 2) {
                    trace("Setting tabChildren to true");
                    stage.tabChildren = true;
                    printProps();
                    trace("Adding a child at 0 and setting tabChildren to false");
                    stage.addChildAt(clip3, 0);
                    stage.tabChildren = false;
                    printProps();
                }
            }
        });
    }

    private function printProps():void {
        trace("stage.tabChildren = " + stage.tabChildren);
        trace("root.tabChildren = " + MovieClip(root).tabChildren);
        trace("clip1.tabChildren = " + clip1.tabChildren);
        trace("clip2.tabChildren = " + clip2.tabChildren);
        trace("clip3.tabChildren = " + clip3.tabChildren);
    }
}
}
