package {
    public class Base {
        private var field:String = "base_private";

        public function Base() {
            trace("///Base()");
            trace(this.field);
            trace(this["field"]);
        }

        public function getFieldBracket():String {
            return this["field"];
        }

        public function getFieldDirect():String {
            return this.field;
        }
    }
}
