package flash.media {
    [API("688")]
    public class AVPeriodInfo {
        private var _localStartTime:Number;
        private var _virtualStartTime:Number;
        private var _duration:Number;
        private var _firstCuePointIndex:int;
        private var _lastCuePointIndex:int;
        private var _firstSubscribedTagIndex:int;
        private var _lastSubscribedTagIndex:int;
        private var _userData:int;
        private var _supportsTrickPlay:Boolean;
        private var _targetDuration:Number;

        public function AVPeriodInfo(
            localStartTime:Number,
            virtualStartTime:Number,
            duration:Number,
            firstCuePointIndex:int,
            lastCuePointIndex:int,
            firstSubscribedTagIndex:int,
            lastSubscribedTagIndex:int,
            userData:int,
            supportsTrickPlay:Boolean,
            targetDuration:Number
        ) {
            this._localStartTime = localStartTime;
            this._virtualStartTime = virtualStartTime;
            this._duration = duration;
            this._firstCuePointIndex = firstCuePointIndex;
            this._lastCuePointIndex = lastCuePointIndex;
            this._firstSubscribedTagIndex = firstSubscribedTagIndex;
            this._lastSubscribedTagIndex = lastSubscribedTagIndex;
            this._userData = userData;
            this._supportsTrickPlay = supportsTrickPlay;
            this._targetDuration = targetDuration;
        }

        public function get localStartTime():Number {
            return this._localStartTime;
        }

        public function get virtualStartTime():Number {
            return this._virtualStartTime;
        }

        public function get duration():Number {
            return this._duration;
        }

        public function get firstCuePointIndex():int {
            return this._firstCuePointIndex;
        }

        public function get lastCuePointIndex():int {
            return this._lastCuePointIndex;
        }

        public function get firstSubscribedTagIndex():int {
            return this._firstSubscribedTagIndex;
        }

        public function get lastSubscribedTagIndex():int {
            return this._lastSubscribedTagIndex;
        }

        public function get userData():int {
            return this._userData;
        }

        public function get supportsTrickPlay():Boolean {
            return this._supportsTrickPlay;
        }

        public function get targetDuration():Number {
            return this._targetDuration;
        }
    }
}
