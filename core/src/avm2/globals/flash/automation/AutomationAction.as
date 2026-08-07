package flash.automation {
    public class AutomationAction {
        private var _type:String = "";

        public function get type():String {
            return this._type;
        }

        public function set type(value:String):void {
            this._type = value;
        }
    }
}
