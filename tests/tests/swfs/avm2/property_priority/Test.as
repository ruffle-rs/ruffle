package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            trace("///new Child()");
            var child:Child = new Child();
            trace("");
            trace("///new Sub2()");
            var sub2:Sub2 = new Sub2();
        }
    }
}
