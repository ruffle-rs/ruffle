package flash.events {
    public class DRMReturnVoucherErrorEvent extends ErrorEvent {
        public static const RETURN_VOUCHER_ERROR:String = "returnVoucherError";

        private var _subErrorID: int;
        private var _serverURL: String;
        private var _licenseID: String;
        private var _policyID: String;

        public function DRMReturnVoucherErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, inDetail:String = "",
            inErrorID:int = 0, inSubErrorID:int = 0, inServerURL:String = null, inLicenseID:String = null, inPolicyID:String = null) {
            super(type, bubbles, cancelable, inDetail, inErrorID);
            this.subErrorID = inSubErrorID;
            this.serverURL = inServerURL;
            this.licenseID = inLicenseID;
            this.policyID = inPolicyID;
        }

        public function get subErrorID():int {
            return this._subErrorID;
        }
        public function set subErrorID(value:int):void {
            this._subErrorID = value;
        }

        public function get serverURL():String {
            return this._serverURL;
        }
        public function set serverURL(value:String):void {
            this._serverURL = value;
        }

        public function get licenseID():String {
            return this._licenseID;
        }
        public function set licenseID(value:String):void {
            this._licenseID = value;
        }

        public function get policyID():String {
            return this._policyID;
        }
        public function set policyID(value:String):void {
            this._policyID = value;
        }

        override public function clone():Event {
            return new DRMReturnVoucherErrorEvent(this.type, this.bubbles, this.cancelable, this.text, this.errorID, this.subErrorID, this.serverURL, this.licenseID, this.policyID);
        }
    }
}
