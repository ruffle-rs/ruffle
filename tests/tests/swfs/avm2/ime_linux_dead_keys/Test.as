package {
import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.type = "input";
        addChild(tf);

        tf.addEventListener(TextEvent.TEXT_INPUT, function(evt:TextEvent):void{
            trace("textInput " + evt.text);
        });
        tf.addEventListener(KeyboardEvent.KEY_DOWN, function(evt:KeyboardEvent):void{
            trace("keyDown " + evt.keyCode + " " + evt.charCode);
        });
        tf.addEventListener(KeyboardEvent.KEY_UP, function(evt:KeyboardEvent):void{
            trace("keyUp " + evt.keyCode + " " + evt.charCode);
        });

        stage.focus = tf;
    }
}
}
