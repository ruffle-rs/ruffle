package {
    import pkg.Base2;
    public class Sub2 extends Base2 {
        public var value:String = "sub2_public";

        public function Sub2() {
            trace("///Sub2()");
            trace(this.value);
            trace(this["value"]);
            trace(getValueBracket());
            trace(getValueDirect());
        }
    }
}
