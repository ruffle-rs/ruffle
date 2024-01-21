package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() { }
    }
}

import flash.net.*;

try {
    getClassByAlias("toString");
} catch (e) {
    trace(e);
}

try {
    getClassByAlias("MyClass");
} catch (e) {
    trace(e);
}

final class TestClass {
};

registerClassAlias("MyClass", TestClass);
trace(getClassByAlias("MyClass"));
