package {
import flash.display.*;
import flash.text.*;
import flash.events.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        var field = new TextField();
        var tf = new TextFormat("TestFont", 10);
        tf.leading = 5;
        field.defaultTextFormat = tf;
        field.type = "input";
        field.embedFonts = true;

        trace(field.textHeight);
        trace(field.getLineMetrics(0).ascent);
        trace(field.getLineMetrics(0).descent);
        trace(field.getLineMetrics(0).height);
        trace(field.getLineMetrics(0).leading);
        trace(field.getLineMetrics(0).width);
        trace(field.getLineMetrics(0).x);
    }
}
}
