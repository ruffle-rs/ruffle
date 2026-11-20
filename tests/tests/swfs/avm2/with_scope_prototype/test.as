package {
    import flash.display.Sprite;

    public class WithScopePrototypeTest extends Sprite {
        public function WithScopePrototypeTest() {
            trace("BEFORE");
            
            // 1. Проверка объекта (ваш главный фикс)
            var obj:Object = <root/>;
            with (obj) {
                trace(name());
            }

            // 2. Проверка примитива (для покрытия 170-й строки)
            var num:Number = 3.14159;
            with (num) {
                trace("PRIMITIVE"); // Просто выводим строку
            }
            
            trace("AFTER");
        }
    }
}