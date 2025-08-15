package flash.events {
    import flash.media.AVResult;

    [API("688")]
    public class AVStatusEvent extends Event {
        public static const AV_STATUS:String = "avStatus";
        public static const BACKGROUND_MANIFEST_ERROR:String = "BackgroundManifestError";
        public static const BACKGROUND_MANIFEST_WARNING:String = "BackgroundManifestWarning";
        public static const BUFFER_STATE:String = "BufferState";
        public static const DECODER_TYPE:String = "DecoderType";
        public static const DIMENSION_CHANGE:String = "DimensionChange";
        public static const ERROR:String = "Error";
        public static const INSERTION_COMPLETE:String = "InsertionComplete";
        public static const LOAD_COMPLETE:String = "LoadComplete";
        public static const MANIFEST_UPDATE:String = "ManifestUpdate";
        public static const PLAY_STATE:String = "PlayState";
        public static const RENDER_TYPE:String = "RenderType";
        public static const SEEK_COMPLETE:String = "SeekComplete";
        public static const STEP_COMPLETE:String = "StepComplete";
        public static const STREAM_SWITCH:String = "StreamSwitch";
        public static const TRICKPLAY_ENDED:String = "TrickPlayEnded";
        public static const WARNING:String = "Warning";

        private var _notificationType:String;
        private var _result:AVResult;
        private var _description:String;

        public function AVStatusEvent(type:String = "avStatus", bubbles:Boolean = false, cancelable:Boolean = false, notificationType:String = "", result:int = 0, description:String = "") {
            super(type, bubbles, cancelable);

            this._notificationType = notificationType;
            this._result = new AVResult(result);
            this._description = description;
        }

        public function get notificationType():String {
            return this._notificationType;
        }

        public function get result():AVResult {
            return this._result;
        }

        public function get description():String {
            return this._description;
        }
    }
}
