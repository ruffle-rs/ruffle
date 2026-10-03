package {
import flash.display.DisplayObject;
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.text.TextField;
import flash.utils.getQualifiedClassName;

public class Test extends Sprite {
    private var tf:TextField;
    private var loader:Loader;

    public function Test() {
        tf = new TextField();

        tf.htmlText =
            "A<img id='probe' src='test_image.jpg'>B";

        var first:DisplayObject = tf.getImageReference("probe");
        var second:DisplayObject = tf.getImageReference("probe");

        trace("REF_NULL=" + (first == null));
        trace("REF_CLASS=" + getQualifiedClassName(first));
        trace("REF_IS_LOADER=" + (first is Loader));
        trace("REF_NAME=" + first.name);
        trace("REF_PARENT_NULL=" + (first.parent == null));
        trace("REF_SAME=" + (first === second));
        trace("UNKNOWN_NULL=" +
            (tf.getImageReference("missing") == null));

        loader = first as Loader;

        trace("INITIAL_WIDTH=" + loader.width);
        trace("INITIAL_HEIGHT=" + loader.height);
        trace("INITIAL_CONTENT_NULL=" +
            (loader.content == null));

        loader.contentLoaderInfo.addEventListener(
            Event.COMPLETE,
            onComplete
        );
    }

    private function onComplete(event:Event):void {
        trace("COMPLETE");
        trace("COMPLETE_REF_SAME=" +
            (tf.getImageReference("probe") === loader));
        trace("COMPLETE_PARENT_NULL=" +
            (loader.parent == null));
        trace("COMPLETE_WIDTH=" + loader.width);
        trace("COMPLETE_HEIGHT=" + loader.height);
        trace("COMPLETE_CONTENT_NULL=" +
            (loader.content == null));

        if (loader.content != null) {
            trace("COMPLETE_CONTENT_CLASS=" +
                getQualifiedClassName(loader.content));
        }
    }
}
}
