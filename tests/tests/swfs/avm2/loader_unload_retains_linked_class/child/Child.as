package {
    import flash.display.MovieClip;

    // Linked by SymbolClass to a sprite that places a 100x100 shape, so that
    // instantiating it has to find the sprite's character - and the shape's -
    // in this movie's library. That lookup is exactly what fails if the
    // library has been released while the class is still alive.
    public class Child extends MovieClip {
        public function Child() {
        }
    }
}
