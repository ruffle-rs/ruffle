package flash.net.drm {
    [API("667")]
    public final class DRMPlaybackTimeWindow {
        private var _period:uint = 0;
        private var _startDate:Date = null;
        private var _endDate:Date = null;

        public function get period():uint {
            return this._period;
        }

        public function get startDate():Date {
            return this._startDate;
        }

        public function get endDate():Date {
            return this._endDate;
        }
    }
}
