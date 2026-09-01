package {
    import flash.display.Sprite;

    // Deliberately quiet: the parent's trace output is what the test compares,
    // and this class exists so that the child SWF carries an ABC and a
    // SymbolClass entry, which is what pins a loaded movie when the class
    // registry holds classes strongly.
    public class Child extends Sprite {
        public function Child() {
        }
    }
}
