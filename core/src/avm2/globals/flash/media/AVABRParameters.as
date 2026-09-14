package flash.media {
    [API("688")]
    public class AVABRParameters {
        public static const AGGRESSIVE:String = "aggressive";
        public static const CONSERVATIVE:String = "conservative";
        public static const MODERATE:String = "moderate";

        private var _policy:String;
        private var _startBitsPerSecond:int;
        private var _minBitsPerSecond:int;
        private var _maxBitsPerSecond:int;

        public function AVABRParameters(
            policy:String,
            startBitsPerSecond:int,
            minBitsPerSecond:int,
            maxBitsPerSecond:int
        ) {
            this._policy = policy;
            this._startBitsPerSecond = startBitsPerSecond;
            this._minBitsPerSecond = minBitsPerSecond;
            this._maxBitsPerSecond = maxBitsPerSecond;
        }

        public function get policy():String {
            return this._policy;
        }

        public function set policy(value:String):void {
            this._policy = value;
        }

        public function get startBitsPerSecond():int {
            return this._startBitsPerSecond;
        }

        public function set startBitsPerSecond(value:int):void {
            this._startBitsPerSecond = value;
        }

        public function get minBitsPerSecond():int {
            return this._minBitsPerSecond;
        }

        public function set minBitsPerSecond(value:int):void {
            this._minBitsPerSecond = value;
        }

        public function get maxBitsPerSecond():int {
            return this._maxBitsPerSecond;
        }

        public function set maxBitsPerSecond(value:int):void {
            this._maxBitsPerSecond = value;
        }
    }
}
