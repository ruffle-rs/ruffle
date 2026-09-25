package {
    public class MyClass {
        public var _theProp:*;

        public function get theProp():* {
            trace("get _theProp called");
            return this._theProp;
        }

        public function set theProp(value:*):void {
            trace("set _theProp called with value " + value);
        }
    }
}
