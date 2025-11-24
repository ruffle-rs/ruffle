package flash.events {
    import flash.text.ime.IIMEClient;

    public class IMEEvent extends TextEvent {
        public static const IME_COMPOSITION:String = "imeComposition";

        [API("667")]
        public static const IME_START_COMPOSITION:String = "imeStartComposition";

        private var _imeClient:IIMEClient;

        public function IMEEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            text:String = "",
            imeClient:IIMEClient = null
        ) {
            super(type, bubbles, cancelable, text);
            this._imeClient = imeClient;
        }

        [API("667")]
        public function get imeClient():IIMEClient {
            return this._imeClient;
        }
        [API("667")]
        public function set imeClient(client:IIMEClient):void {
            this._imeClient = client;
        }

        override public function clone():Event {
            return new IMEEvent(type, bubbles, cancelable, text, this._imeClient);
        }

        override public function toString():String {
            return formatToString(
                "IMEEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "text",
                "imeClient");
        }
    }
}
