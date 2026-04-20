package {
import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.type = "input";
        tf.width = 200;
        tf.height = 800;
        tf.text = "abcd";
        addChild(tf);

        tf.addEventListener(TextEvent.TEXT_INPUT, function(evt:TextEvent):void{
            trace("textInput " + evt.text + ", " + tf.text);
        });
        tf.addEventListener(Event.CHANGE, function(evt:*):void{
            trace("change " + tf.text);
        });
        stage.addEventListener(KeyboardEvent.KEY_DOWN, function(evt:KeyboardEvent):void{
            trace("keyDown " + evt.keyCode + ", " + evt.charCode + ", " + tf.text);
        });
        stage.addEventListener(KeyboardEvent.KEY_UP, function(evt:KeyboardEvent):void{
            trace("keyUp " + evt.keyCode + ", " + evt.charCode + ", " + tf.text);
        });

        stage.focus = tf;
        tf.setSelection(2,2);
    }
}
}
