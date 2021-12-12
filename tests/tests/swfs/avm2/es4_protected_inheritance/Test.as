// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }

    }

}

class A {
    public function a(): void { foo(); }
    protected function foo(): void { trace("A::foo")}
}

class B extends A {
    public function b(): void { foo(); }
}

class C extends B {
    public function c(): void { foo(); }
    override protected function foo(): void {
        trace("C::foo");
        super.foo();
    }
}

var c: C = new C();
c.a();
c.b();
c.c();
