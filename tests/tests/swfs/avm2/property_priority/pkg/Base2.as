package pkg {
    public class Base2 {
        internal var value:String = "base2_internal";

        public function Base2() {
            trace("///Base2()");
            trace(this.value);
            trace(this["value"]);
        }

        public function getValueBracket():String {
            return this["value"];
        }

        public function getValueDirect():String {
            return this.value;
        }
    }
}
