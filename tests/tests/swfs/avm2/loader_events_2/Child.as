package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Child extends MovieClip {
        public function Child() {
            trace("child");

            addEventListener(Event.ADDED, onEvent);
            addEventListener(Event.ADDED_TO_STAGE, onEvent);
            addEventListener(Event.REMOVED, onEvent);
            addEventListener(Event.REMOVED_FROM_STAGE, onEvent);
            loaderInfo.addEventListener(Event.UNLOAD, onEvent);

            stop();
        }

        private function onEvent(event:Event):void {
            trace(
                "child " +
                event.type +
                ": target=" + event.target +
                ", currentTarget=" + event.currentTarget +
                ", eventPhase=" + event.eventPhase +
                ", bubbles=" + event.bubbles
            );
        }
    }
}
