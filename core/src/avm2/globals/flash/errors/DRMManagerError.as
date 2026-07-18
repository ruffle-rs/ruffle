package flash.errors {
    [API("667")]
    public class DRMManagerError extends Error {
        // NOTE: Unlike all the other `flash.errors` classes, `DRMManagerError`
        // doesn't set its `prototype.name`

        private var _subErrorID:int;

        public function DRMManagerError(message:String, id:int, subErrorID:int) {
            super(message, id);
            this._subErrorID = subErrorID;
        }

        public function get subErrorID():int {
            return this._subErrorID;
        }

        public function toString():String {
            return "DRMManagerError: '" + this.message + "', error ID:'" + this.errorID.toString() + "', subErrorID:'" + this._subErrorID.toString() + "'";
        }
    }
}
