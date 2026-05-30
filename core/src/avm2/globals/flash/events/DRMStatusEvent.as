package flash.events {
    import flash.net.drm.DRMContentData;
    import flash.net.drm.DRMVoucher;

    [API("667")]
    public class DRMStatusEvent extends Event {
        public static const DRM_STATUS:String = "drmStatus";

        private var _contentData:DRMContentData;
        private var _voucher:DRMVoucher;
        private var _isLocal:Boolean;

        public function DRMStatusEvent(type:String = "drmStatus", bubbles:Boolean = false, cancelable:Boolean = false, contentData:DRMContentData = null, voucher:DRMVoucher = null, local:Boolean = false) {
            super(type, bubbles, cancelable);

            this._contentData = contentData;
            this._voucher = voucher;
            this._isLocal = local;
        }

        public function get contentData():DRMContentData {
            return this._contentData;
        }
        public function set contentData(value:DRMContentData):void {
            this._contentData = value;
        }

        public function get voucher():DRMVoucher {
            return this._voucher;
        }
        public function set voucher(value:DRMVoucher):void {
            this._voucher = value;
        }

        public function get isLocal():Boolean {
            return this._isLocal;
        }
        public function set isLocal(value:Boolean):void {
            this._isLocal = value;
        }

        override public function clone():Event {
            return new DRMStatusEvent(type, bubbles, cancelable, this._contentData, this._voucher, this._isLocal);
        }

        override public function toString():String {
            return this.formatToString("DRMStatusEvent", "type", "bubbles", "cancelable");
        }
    }
}
