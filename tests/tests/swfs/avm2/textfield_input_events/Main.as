package {
    import flash.display.MovieClip;
    import flash.display.DisplayObject;
    import flash.display.InteractiveObject;

    public class Main extends MovieClip {
        public function Main() {
            this.getChildAt(0).addEventListener("change", function(e) {
                trace("New text: " + e.target.text);
            });
            this.getChildAt(0).addEventListener("textInput", function(e) {
                trace("Text input: " + e.text);
                if (e.text == "#") {
                    trace("Prevented default!");
                    e.preventDefault();
                }
            });
            this.stage.focus = this.getChildAt(0) as InteractiveObject;
        }
    }
}
