package flash.events {
    public class TextEvent extends Event {
        public static const LINK:String = "link";
        public static const TEXT_INPUT:String = "textInput";

        private var _text:String;

        public function TextEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            text:String = ""
        ) {
            super(type, bubbles, cancelable);
            this.text = text;
        }

        public function get text():String {
            return this._text;
        }
        public function set text(value:String):void {
            this._text = value;
        }

        override public function clone():Event {
            return new TextEvent(this.type, this.bubbles, this.cancelable, this.text);
        }

        override public function toString():String {
            return this.formatToString(
                "TextEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "text"
            );
        }
    }
}
