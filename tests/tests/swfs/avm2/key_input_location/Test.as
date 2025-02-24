package {
import flash.display.*;
import flash.text.*;
import flash.text.engine.*;
import flash.events.*;
import flash.geom.*;
import flash.utils.getTimer;

[SWF(frameRate="25")]
public class Test extends MovieClip {
    public function Test() {
        stage.addEventListener("keyDown", function(evt: KeyboardEvent): void {
            trace("Key down:");
            printKey(evt);
        });
        stage.addEventListener("keyUp", function(evt: KeyboardEvent): void {
            trace("Key up:");
            printKey(evt);
        });
    }

    private function printKey(evt: KeyboardEvent): void {
        trace("  altKey: " + evt.altKey);
        trace("  charCode: " + evt.charCode);
        trace("  ctrlKey: " + evt.ctrlKey);
        trace("  keyCode: " + evt.keyCode);
        trace("  keyLocation: " + evt.keyLocation);
        trace("  shiftKey: " + evt.shiftKey);
    }
}
}
