package {
import flash.display.*;
import flash.text.*;

public class Test extends Sprite {
    public function Test() {
        var tf = new TextFormat("some  very,very,very,very,very,very,very,very,very,very,very,very,very, long font");
        trace("Constructor: " + tf.font);
        tf.font = "some 2 very,very,very,very,very,very,very,very,very,very,very,very,very, long font";
        trace("Setter: " + tf.font);
        tf.font = "some not too long font";
        trace("Setter 2: " + tf.font);

        var field: TextField = new TextField();
        field.htmlText = "<font face='some 3 very,very,very,very,very,very,very,very,very,very,very,very,very, long font'>x</font>";
        trace("HTML: " + field.getTextFormat(0, 1).font);
    }
}
}
