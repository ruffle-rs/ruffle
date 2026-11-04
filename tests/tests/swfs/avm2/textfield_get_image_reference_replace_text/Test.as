package {
import flash.display.DisplayObject;
import flash.display.Loader;
import flash.display.Sprite;
import flash.text.TextField;

public class Test extends Sprite {
    public function Test() {
        var tf:TextField = new TextField();

        tf.htmlText =
            "A<img id='first' src='test_image.jpg'>" +
            "B<img id='second' src='test_image.jpg'>C";

        var firstBefore:DisplayObject =
            tf.getImageReference("first");

        var secondBefore:DisplayObject =
            tf.getImageReference("second");

        trace("TEXT_BEFORE=[" + tf.text + "]");
        trace("LENGTH_BEFORE=" + tf.length);
        trace("FIRST_BEFORE_NULL=" + (firstBefore == null));
        trace("SECOND_BEFORE_NULL=" + (secondBefore == null));

        // Character 1 is the placeholder for the first IMG.
        tf.replaceText(1, 2, "");

        var firstAfter:DisplayObject =
            tf.getImageReference("first");

        var secondAfter:DisplayObject =
            tf.getImageReference("second");

        trace("TEXT_AFTER=[" + tf.text + "]");
        trace("LENGTH_AFTER=" + tf.length);

        trace("FIRST_AFTER_NULL=" +
            (firstAfter == null));

        trace("SECOND_AFTER_NULL=" +
            (secondAfter == null));

        trace("SECOND_SAME=" +
            (secondBefore === secondAfter));

        trace("SECOND_BECAME_FIRST=" +
            (firstBefore === secondAfter));
    }
}
}
