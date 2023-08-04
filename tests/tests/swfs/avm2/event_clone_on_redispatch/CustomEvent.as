package {
    import flash.events.Event;

    public class CustomEvent extends Event {
        public function CustomEvent(type:String, bubbles:Boolean, cancelable:Boolean) {
            super(type, bubbles, cancelable);
        }

        override public function clone():Event {
            trace("CustomEvent cloned!");
            return new CustomEvent(type, bubbles, cancelable);
        }
    }

}