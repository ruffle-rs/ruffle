package flash.media {
    import flash.utils.Dictionary;

    [API("688")]
    public class AVCuePoint {
        private var _dictionary:Dictionary;
        private var _localTime:Number;

        public function AVCuePoint(
            dictionary:Dictionary,
            localTime:Number
        ) {
            this._dictionary = dictionary;
            this._localTime = localTime;
        }

        public function get dictionary():Dictionary {
            return this._dictionary;
        }

        public function get localTime():Number {
            return this._localTime;
        }
    }
}
