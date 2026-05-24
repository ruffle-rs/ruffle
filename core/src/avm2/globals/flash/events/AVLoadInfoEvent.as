package flash.events {
    import flash.utils.Dictionary;

    [API("688")]
    public class AVLoadInfoEvent extends Event {
        public static const AV_LOAD_INFO = "avLoadInfo";

        private var _loadInfo:Dictionary;

        public function AVLoadInfoEvent(type:String = "avLoadInfo", bubbles:Boolean = false, cancelable:Boolean = false, loadInfo:Dictionary = null) {
            super(type, bubbles, cancelable);
            this._loadInfo = loadInfo;
        }

        public function get loadInfo():Dictionary {
            return this._loadInfo;
        }
    }
}
