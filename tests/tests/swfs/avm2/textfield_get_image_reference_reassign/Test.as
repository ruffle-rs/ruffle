package {
import flash.display.DisplayObject;
import flash.display.Sprite;
import flash.text.TextField;

public class Test extends Sprite {
    public function Test() {
        testHtmlTextReassign();
        testTextReassign();
    }

    private function testHtmlTextReassign():void {
        var tf:TextField = new TextField();
        var html:String =
            "A<img id='probe' src='test_image.jpg'>B";

        tf.htmlText = html;

        var first:DisplayObject =
            tf.getImageReference("probe");

        tf.htmlText = html;

        var second:DisplayObject =
            tf.getImageReference("probe");

        trace("HTML_FIRST_NULL=" + (first == null));
        trace("HTML_SECOND_NULL=" + (second == null));
        trace("HTML_SAME_REFERENCE=" + (first === second));
    }

    private function testTextReassign():void {
        var tf:TextField = new TextField();

        tf.htmlText =
            "A<img id='probe' src='test_image.jpg'>B";

        var before:DisplayObject =
            tf.getImageReference("probe");

        var text:String = tf.text;
        tf.text = text;

        var after:DisplayObject =
            tf.getImageReference("probe");

        trace("TEXT_BEFORE_NULL=" + (before == null));
        trace("TEXT_AFTER_NULL=" + (after == null));
        trace("TEXT_SAME_REFERENCE=" + (before === after));
    }
}
}
