package {
    import flash.display.Sprite;

    public class WithScopePrototypeTest extends Sprite {
        public function WithScopePrototypeTest() {
            // Статический тип Object — mxmlc не сможет доказать,
            // что name() есть у obj, и обязан сгенерировать
            // pushwith + findpropstrict name + callproperty name, 0.
            trace("BEFORE"); // <- маркер
            var obj:Object = <root/>;
            with (obj) {
                trace(name()); // name() на XML.prototype → без фикса #1006
            }
            trace("AFTER"); // <- маркер
        }
    }
}