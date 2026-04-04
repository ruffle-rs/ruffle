package {
    public class Child extends Base {
        public var field:String = "child_public";

        public function Child() {
            trace("///Child pre-super");
            trace(this.field);
            trace(this["field"]);
            super();
            trace("///Child post-super");
            trace(this.field);
            trace(this["field"]);
            trace(getFieldBracket());
            trace(getFieldDirect());
        }
    }
}
