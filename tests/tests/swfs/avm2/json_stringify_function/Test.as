package {
import flash.display.Sprite;

public class Test extends Sprite {
    function Test() {
        test(Object)
        test(escape);
        test(this.test);

        test({ a: Object });
        test({ a: escape });
        test({ a: this.test });

        test({ a: Object, b: true });
        test({ a: escape, b: true });
        test({ a: this.test, b: true });

        test([1, Object, 3]);
        test([1, escape, 3]);
        test([1, this.test, 3]);
    }

    private function test(value: *): void {
        trace(JSON.stringify(value));
    }
}

}
