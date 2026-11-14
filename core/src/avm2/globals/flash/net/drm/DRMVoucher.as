package flash.net.drm {
    import flash.utils.ByteArray;

    [API("667")]
    public final class DRMVoucher {
        private var _licenseStartDate:Date = null;
        private var _licenseEndDate:Date = null;
        private var _offlineLeaseStartDate:Date = null;
        private var _offlineLeaseEndDate:Date = null;
        private var _policies:Object = null;
        private var _playbackTimeWindow:DRMPlaybackTimeWindow = null;

        public function get licenseStartDate():Date {
            return this._licenseStartDate;
        }

        public function get licenseEndDate():Date {
            return this._licenseEndDate;
        }

        public function get offlineLeaseStartDate():Date {
            return this._offlineLeaseStartDate;
        }

        public function get offlineLeaseEndDate():Date {
            return this._offlineLeaseEndDate;
        }

        public function get policies():Object {
            return this._policies;
        }

        public function get playbackTimeWindow():DRMPlaybackTimeWindow {
            return this._playbackTimeWindow;
        }

        [API("690")]
        public function toByteArray():ByteArray {
            return new ByteArray();
        }
    }
}
