package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            trace(("ab" + "c").AS3::toString === "abc".AS3::toString);
            trace(("ab" + "c").AS3::toString === ("ab" + "c").AS3::toString);
            trace("abc".AS3::toString === "abc".AS3::toString);
            trace("1".AS3::toString === "1".AS3::toString);
            trace("".AS3::toString === "".AS3::toString);
            trace((true).AS3::toString === (true).AS3::toString);
            trace((false).AS3::toString === (false).AS3::toString);
            trace((false).AS3::toString === (true).AS3::toString);
            trace((true).AS3::toString === (false).AS3::toString);
            trace((true).AS3::toString === "true".AS3::toString);
            trace("1".AS3::toString === (1).AS3::toString);
        }
    }
}
