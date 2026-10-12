package  {
    import flash.display.MovieClip;
    import flash.events.MouseEvent;

    public class Test extends MovieClip {
        public function Test() {
            stage.addEventListener(MouseEvent.MOUSE_DOWN, onMouseDown);
            stage.addEventListener(MouseEvent.MOUSE_UP, onMouseUp);
            stage.addEventListener(MouseEvent.MOUSE_MOVE, onMouseMove);
            trace("Loaded!");
        }

        function onMouseDown(event: MouseEvent): void {
            trace("onMouseDown(" + Math.round(event.stageX) + "," + Math.round(event.stageY) + ")");
        }

        function onMouseUp(event: MouseEvent): void {
            trace("onMouseUp(" + Math.round(event.stageX) + "," + Math.round(event.stageY) + ")");
        }

        function onMouseMove(event: MouseEvent): void {
            trace("onMouseMove(" + Math.round(event.stageX) + "," + Math.round(event.stageY) + ")");
        }
    }
}
