package  {
    import flash.display.MovieClip;
    import flash.events.KeyboardEvent;

    public class Test extends MovieClip {
        public function Test() {
            stage.addEventListener(KeyboardEvent.KEY_DOWN, onKeyDown);
            stage.addEventListener(KeyboardEvent.KEY_UP, onKeyUp);

            trace("Loaded!");
        }

        function onKeyDown(event: KeyboardEvent) {
            trace("onKeyDown(" + event.charCode + "," + event.keyCode + ")");
        }

        function onKeyUp(event: KeyboardEvent) {
            trace("onKeyUp(" + event.charCode + "," + event.keyCode + ")");
        }
    }
}
