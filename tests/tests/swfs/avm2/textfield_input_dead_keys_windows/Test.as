package {
import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends MovieClip {
    public function Test() {
        var tf1 = new TextField();
        tf1.border = true;
        tf1.type = "input";
        tf1.width = 200;
        tf1.height = 200;
        addChild(tf1);

        tf1.addEventListener(TextEvent.TEXT_INPUT, function(evt:TextEvent):void{
            trace("textInput " + evt.text);
        });
        stage.addEventListener(KeyboardEvent.KEY_DOWN, function(evt:KeyboardEvent):void{
            trace("keyDown " + evt.keyCode + " " + evt.charCode);
        });
        stage.addEventListener(KeyboardEvent.KEY_UP, function(evt:KeyboardEvent):void{
            trace("keyUp " + evt.keyCode + " " + evt.charCode);
        });

        stage.focus = tf1;
    }
}
}
