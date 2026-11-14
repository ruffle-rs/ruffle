package flash.media {
    [API("674")]
    public class H264VideoStreamSettings extends VideoStreamSettings {
        private var _profile:String = H264Profile.BASELINE;
        private var _level:String = H264Level.LEVEL_2_1;

        public function H264VideoStreamSettings() {
            super();
        }

        override public function get codec():String {
            return VideoCodec.H264AVC;
        }

        public function get level():String {
            return this._level;
        }

        public function get profile():String {
            return this._profile;
        }

        public function setProfileLevel(profile:String, level:String):void {
            if (profile != H264Profile.BASELINE && profile != H264Profile.MAIN) {
                Error.throwError(ArgumentError, 2008, "profile");
            }
            this._profile = profile;
            this._level = level;
        }
    }
}
