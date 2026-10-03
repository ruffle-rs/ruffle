package {
import flash.display.DisplayObject;
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.IOErrorEvent;
import flash.text.TextField;

public class Test extends Sprite {
    private var duplicateLoader:Loader;

    public function Test() {
        testDuplicateId();
        testEmptyId();
        testCaseSensitivity();
        testSameSrcDifferentIds();
        testReplaceHtml();
        testTextSetter();
        testMissingSrc();
        testEmptySrc();
    }

    private function makeField(html:String):TextField {
        var tf:TextField = new TextField();
        tf.htmlText = html;
        return tf;
    }

    private function testDuplicateId():void {
        trace("=== DUPLICATE_ID ===");

        var tf:TextField = makeField(
            "A<img id='dup' src='dup_first.jpg'>" +
            "B<img id='dup' src='dup_second.jpg'>C"
        );

        var a:DisplayObject = tf.getImageReference("dup");
        var b:DisplayObject = tf.getImageReference("dup");

        trace("DUP_NULL=" + (a == null));
        trace("DUP_SAME_LOOKUP=" + (a === b));

        if (a != null) {
            trace("DUP_NAME=" + a.name);
        }

        duplicateLoader = a as Loader;

        if (duplicateLoader != null) {
            duplicateLoader.contentLoaderInfo.addEventListener(
                Event.COMPLETE,
                onDuplicateComplete
            );
        }
    }

    private function testEmptyId():void {
        trace("=== EMPTY_ID ===");

        var tf:TextField = makeField(
            "A<img id='' src='test_image.jpg'>B"
        );

        var ref:DisplayObject = tf.getImageReference("");

        trace("EMPTY_ID_NULL=" + (ref == null));

        if (ref != null) {
            trace("EMPTY_ID_NAME=[" + ref.name + "]");
        }
    }

    private function testCaseSensitivity():void {
        trace("=== CASE_SENSITIVITY ===");

        var tf:TextField = makeField(
            "A<img id='Probe' src='test_image.jpg'>B"
        );

        trace(
            "CASE_EXACT_NULL=" +
            (tf.getImageReference("Probe") == null)
        );

        trace(
            "CASE_LOWER_NULL=" +
            (tf.getImageReference("probe") == null)
        );
    }

    private function testSameSrcDifferentIds():void {
        trace("=== SAME_SRC_DIFFERENT_IDS ===");

        var tf:TextField = makeField(
            "A<img id='first' src='test_image.jpg'>" +
            "B<img id='second' src='test_image.jpg'>C"
        );

        var first:DisplayObject =
            tf.getImageReference("first");

        var second:DisplayObject =
            tf.getImageReference("second");

        trace("SAME_SRC_FIRST_NULL=" + (first == null));
        trace("SAME_SRC_SECOND_NULL=" + (second == null));
        trace(
            "SAME_SRC_DIFFERENT_OBJECTS=" +
            (first !== second)
        );
    }

    private function testReplaceHtml():void {
        trace("=== REPLACE_HTML ===");

        var tf:TextField = makeField(
            "A<img id='old' src='test_image.jpg'>B"
        );

        var oldRef:DisplayObject =
            tf.getImageReference("old");

        trace("OLD_BEFORE_NULL=" + (oldRef == null));

        tf.htmlText =
            "A<img id='new' src='test_image.jpg'>B";

        var oldAfter:DisplayObject =
            tf.getImageReference("old");

        var newRef:DisplayObject =
            tf.getImageReference("new");

        trace("OLD_AFTER_NULL=" + (oldAfter == null));
        trace("NEW_AFTER_NULL=" + (newRef == null));
        trace(
            "OLD_NEW_DIFFERENT=" +
            (oldRef !== newRef)
        );

        tf.htmlText = "plain text";

        trace(
            "NEW_AFTER_REMOVE_NULL=" +
            (tf.getImageReference("new") == null)
        );
    }

    private function testTextSetter():void {
        trace("=== TEXT_SETTER_AFTER_HTML ===");

        var tf:TextField = makeField(
            "A<img id='textTest' src='test_image.jpg'>B"
        );

        trace(
            "TEXT_SETTER_BEFORE_NULL=" +
            (tf.getImageReference("textTest") == null)
        );

        tf.text = "plain";

        trace(
            "TEXT_SETTER_AFTER_NULL=" +
            (tf.getImageReference("textTest") == null)
        );
    }

    private function testMissingSrc():void {
        trace("=== MISSING_SRC ===");

        var tf:TextField = makeField(
            "A<img id='noSrc'>B"
        );

        var ref:DisplayObject =
            tf.getImageReference("noSrc");

        trace("MISSING_SRC_NULL=" + (ref == null));

        if (ref != null) {
            trace("MISSING_SRC_CLASS=" +
                Object(ref).constructor);
            trace("MISSING_SRC_NAME=" + ref.name);
        }
    }

    private function testEmptySrc():void {
        trace("=== EMPTY_SRC ===");

        var tf:TextField = makeField(
            "A<img id='emptySrc' src=''>B"
        );

        var ref:DisplayObject =
            tf.getImageReference("emptySrc");

        var loader:Loader = ref as Loader;
        if (loader != null) {
            loader.contentLoaderInfo.addEventListener(
                IOErrorEvent.IO_ERROR,
                ignoreIoError
            );
        }

        trace("EMPTY_SRC_NULL=" + (ref == null));

        if (ref != null) {
            trace("EMPTY_SRC_NAME=" + ref.name);
        }
    }

    private function onDuplicateComplete(event:Event):void {
        var url:String =
            duplicateLoader.contentLoaderInfo.url;

        trace("=== DUPLICATE_COMPLETE ===");
        trace(
            "DUP_URL_IS_FIRST=" +
            (url.indexOf("dup_first.jpg") >= 0)
        );
        trace(
            "DUP_URL_IS_SECOND=" +
            (url.indexOf("dup_second.jpg") >= 0)
        );
    }

    private function ignoreIoError(event:IOErrorEvent):void {
    }
}
}
