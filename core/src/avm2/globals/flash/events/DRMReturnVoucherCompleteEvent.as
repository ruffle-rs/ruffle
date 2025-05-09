package flash.events {
    [API("690")]
    public class DRMReturnVoucherCompleteEvent extends Event {
        public static const RETURN_VOUCHER_COMPLETE:String = "returnVoucherComplete";

        private var _serverURL: String;
        private var _licenseID: String;
        private var _policyID: String;
        private var _numberOfVouchersReturned: int;

        public function DRMReturnVoucherCompleteEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, inServerURL:String = null,
            inLicenseID:String = null, inPolicyID:String = null, inNumberOfVouchersReturned:int = 0) {
            super(type, bubbles, cancelable);
            this.serverURL = inServerURL;
            this.licenseID = inLicenseID;
            this.policyID = inPolicyID;
            this.numberOfVouchersReturned = inNumberOfVouchersReturned;
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

        public function get numberOfVouchersReturned():int {
            return this._numberOfVouchersReturned;
        }
        public function set numberOfVouchersReturned(value:int):void {
            this._numberOfVouchersReturned = value;
        }

        override public function clone():Event {
            return new DRMReturnVoucherCompleteEvent(this.type, this.bubbles, this.cancelable, this.serverURL, this.licenseID, this.policyID, this.numberOfVouchersReturned);
        }
    }
}
