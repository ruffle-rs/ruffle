package flash.events {
    public class DRMLicenseRequestEvent extends Event {
        public static const LICENSE_REQUEST:String = "licenseRequest";

        private var _serverURL: String;

        public function DRMLicenseRequestEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, inServerURL:String = null) {
            super(type,bubbles,cancelable);
            this.serverURL = inServerURL;
        }

        public function get serverURL():String {
            return this._serverURL;
        }
        public function set serverURL(value:String):void {
            this._serverURL = value;
        }

        override public function clone():Event {
            return new DRMLicenseRequestEvent(this.type, this.bubbles, this.cancelable, this.serverURL);
        }
    }
}
