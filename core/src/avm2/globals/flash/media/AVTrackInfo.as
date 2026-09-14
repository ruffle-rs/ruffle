package flash.media {
    [API("688")]
    public class AVTrackInfo {
        public static const DTI_608_CAPTIONS:String = "DTI608Captions";
        public static const DTI_708_CAPTIONS:String = "DTI708Captions";
        public static const DTI_WEBVTT_CAPTIONS:String = "DTIWebVTTCaptions";

        private var _description:String;
        private var _language:String;
        private var _defaultTrack:Boolean;
        private var _autoSelect:Boolean;
        private var _forced:Boolean;
        private var _activity:Boolean;
        private var _dataTrackInfoServiceType:String;
        private var _pid:int;

        public function AVTrackInfo(
            description:String,
            language:String,
            defaultTrack:Boolean,
            autoSelect:Boolean,
            forced:Boolean,
            activity:Boolean,
            dataTrackInfoServiceType:String,
            pid:int
        ) {
            this._description = description;
            this._language = language;
            this._defaultTrack = defaultTrack;
            this._autoSelect = autoSelect;
            this._forced = forced;
            this._activity = activity;
            this._dataTrackInfoServiceType = dataTrackInfoServiceType;
            this._pid = pid;
        }

        public function get description():String {
            return this._description;
        }

        public function get language():String {
            return this._language;
        }

        public function get defaultTrack():Boolean {
            return this._defaultTrack;
        }

        public function get autoSelect():Boolean {
            return this._autoSelect;
        }

        public function get forced():Boolean {
            return this._forced;
        }

        public function get activity():Boolean {
            return this._activity;
        }

        public function get dataTrackInfoServiceType():String {
            return this._dataTrackInfoServiceType;
        }

        public function get pid():int {
            return this._pid;
        }
    }
}
