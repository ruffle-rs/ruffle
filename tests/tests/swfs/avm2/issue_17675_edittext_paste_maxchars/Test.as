// compiled with mxmlc

package {
    import flash.display.MovieClip;
    import flash.text.TextField;
    public class Test extends MovieClip {
        public function Test(){
            var tf = new TextField();
            tf.text = "text";
            tf.maxChars = 5;
            tf.type = "input";
            addChild(tf);
            tf.addEventListener("change", function(){
                trace(tf.text);
            });
            stage.focus = tf;
        }
    }
}
