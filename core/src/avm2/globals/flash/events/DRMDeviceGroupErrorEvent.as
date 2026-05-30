package flash.events {
    import flash.net.drm.DRMDeviceGroup;

    [API("692")]
    public class DRMDeviceGroupErrorEvent extends ErrorEvent {
        public static const ADD_TO_DEVICE_GROUP_ERROR:String = "addToDeviceGroupError";
        public static const REMOVE_FROM_DEVICE_GROUP_ERROR:String = "removeFromDeviceGroupError";

        private var _deviceGroup:DRMDeviceGroup;
        private var _subErrorID:int;
        private var _systemUpdateNeeded:Boolean;
        private var _drmUpdateNeeded:Boolean;

        public function DRMDeviceGroupErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, errorDetail:String = "", errorCode:int = 0, subErrorID:int = 0, deviceGroup:DRMDeviceGroup = null, systemUpdateNeeded:Boolean = false, drmUpdateNeeded:Boolean = false) {
            super(type, bubbles, cancelable, errorDetail, errorCode);
            this._subErrorID = subErrorID;
            this._deviceGroup = deviceGroup;
            this._systemUpdateNeeded = systemUpdateNeeded;
            this._drmUpdateNeeded = drmUpdateNeeded;
        }

        public function get subErrorID():int {
            return this._subErrorID;
        }
        public function set subErrorID(value:int):void {
            this._subErrorID = value;
        }

        public function get deviceGroup():DRMDeviceGroup {
            return this._deviceGroup;
        }
        public function set deviceGroup(value:DRMDeviceGroup) {
            this._deviceGroup = value;
        }

        public function get systemUpdateNeeded():Boolean {
            return this._systemUpdateNeeded;
        }

        public function get drmUpdateNeeded():Boolean {
            return this._drmUpdateNeeded;
        }

        override public function clone():Event {
            return new DRMDeviceGroupErrorEvent(type, bubbles, cancelable, text, errorID, this.subErrorID, this._deviceGroup, this._systemUpdateNeeded, this._drmUpdateNeeded);
        }

        override public function toString():String {
            return this.formatToString("DRMDeviceGroupErrorEvent", "type", "bubbles", "cancelable", "eventPhase", "errorID", "subErrorID", "text", "systemUpdateNeeded", "drmUpdateNeeded");
        }
    }
}
