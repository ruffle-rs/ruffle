package flash.events {
    import flash.net.drm.DRMDeviceGroup;

    [API("692")]
    public class DRMDeviceGroupEvent extends Event {
        public static const ADD_TO_DEVICE_GROUP_COMPLETE:String = "addToDeviceGroupComplete";
        public static const REMOVE_FROM_DEVICE_GROUP_COMPLETE:String = "removeFromDeviceGroupComplete";

        private var _deviceGroup:DRMDeviceGroup;

        public function DRMDeviceGroupEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, deviceGroup:DRMDeviceGroup = null) {
            super(type, bubbles, cancelable);
            this._deviceGroup = deviceGroup;
        }

        public function get deviceGroup():DRMDeviceGroup {
            return this._deviceGroup;
        }
        public function set deviceGroup(value:DRMDeviceGroup) {
            this._deviceGroup = value;
        }

        override public function clone():Event {
            return new DRMDeviceGroupEvent(type, bubbles, cancelable, this._deviceGroup);
        }

        override public function toString():String {
            return this.formatToString("DRMDeviceGroupEvent", "type", "bubbles", "cancelable", "eventPhase", "deviceGroup");
        }
    }
}
