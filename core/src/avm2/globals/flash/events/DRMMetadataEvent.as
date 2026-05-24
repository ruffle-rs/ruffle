package flash.events {
    import flash.net.drm.DRMContentData;
    import flash.utils.ByteArray;

    public class DRMMetadataEvent extends Event {
        public static const DRM_METADATA:String = "drmMetadata";

        private var _drmMetadata:DRMContentData;
        private var _timestamp:Number;

        public function DRMMetadataEvent(type:String = "drmMetadata", bubbles:Boolean = false, cancelable:Boolean = false, metadata:ByteArray = null, timestamp:Number = 0.0) {
            super(type, bubbles, cancelable);
            this._drmMetadata = new DRMContentData(metadata);
            this._timestamp = timestamp;
        }

        public function get drmMetadata():DRMContentData {
            return this._drmMetadata;
        }

        public function get timestamp():Number {
            return this._timestamp;
        }
    }
}
