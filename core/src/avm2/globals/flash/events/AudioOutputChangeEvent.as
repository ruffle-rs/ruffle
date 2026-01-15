package flash.events {
    // TODO: [API("724")]
    public class AudioOutputChangeEvent extends Event {
        public static const AUDIO_OUTPUT_CHANGE:String = "audioOutputChange";

        private var _reason:String;

        public function AudioOutputChangeEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            reason:String = null
        ) {
            super(type, bubbles, cancelable);
            this._reason = reason;
        }

        public function get reason():String {
            return this._reason;
        }
    }
}
