package  {
    import flash.display.MovieClip;
    import flash.text.TextField;
    import flash.text.TextFormat;
    import flash.utils.setTimeout;

    [SWF(width="400", height="400")]
    public class Test extends MovieClip {
        public function Test() {
            var tf = new TextFormat();
            tf.font = "Some Unknown Font 3";
            tf.size = 10;

            var f = new TextField();
            f.defaultTextFormat = tf;
            f.text = "test";
            addChild(f);
            trace("Loaded!");
        }
    }
}
