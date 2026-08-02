package flash.media {
    [API("688")]
    public class AVTimeline {
        private var _type:String;
        private var _virtualStartTime:Number;
        private var _virtualDuration:Number;
        private var _firstPeriodIndex:int;
        private var _lastPeriodIndex:int;
        private var _firstSubscribedTagIndex:int;
        private var _lastSubscribedTagIndex:int;
        private var _complete:Boolean;

        public function AVTimeline(
            type:String,
            virtualStartTime:Number,
            virtualDuration:Number,
            firstPeriodIndex:int,
            lastPeriodIndex:int,
            firstSubscribedTagIndex:int,
            lastSubscribedTagIndex:int,
            complete:Boolean
        ) {
            this._type = type;
            this._virtualStartTime = virtualStartTime;
            this._virtualDuration = virtualDuration;
            this._firstPeriodIndex = firstPeriodIndex;
            this._lastPeriodIndex = lastPeriodIndex;
            this._firstSubscribedTagIndex = firstSubscribedTagIndex;
            this._lastSubscribedTagIndex = lastSubscribedTagIndex;
            this._complete = complete;
        }

        public function get type():String {
            return this._type;
        }

        public function get virtualStartTime():Number {
            return this._virtualStartTime;
        }

        public function get virtualDuration():Number {
            return this._virtualDuration;
        }

        public function get firstPeriodIndex():int {
            return this._firstPeriodIndex;
        }

        public function get lastPeriodIndex():int {
            return this._lastPeriodIndex;
        }

        public function get firstSubscribedTagIndex():int {
            return this._firstSubscribedTagIndex;
        }

        public function get lastSubscribedTagIndex():int {
            return this._lastSubscribedTagIndex;
        }

        public function get complete():Boolean {
            return this._complete;
        }
    }
}
