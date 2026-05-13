package {
    import flash.display.Sprite;

    public class Child extends Sprite {
        public function Child() {
            trace("Hello from loaded SWF!");

            throw new Error("Uncaught error in constructor");
        }
    }
}
