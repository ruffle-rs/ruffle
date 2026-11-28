// The root's class is only linked to the root once its class initializer
// has returned. Here, the class initializer constructs a library clip with a
// button on its timeline. Constructing the button runs a nested frame, which
// mustn't construct the root yet (it would be a plain MovieClip): the root is
// constructed afterwards, as an instance of Main.
package {
    import flash.display.MovieClip;

    public class Main extends MovieClip {
        public static var holder:Holder = new Holder();

        public function Main() {
            trace("Main: constructor, root is Main: " + (root is Main));
            trace("holder: " + holder + ", numChildren: " + holder.numChildren);
            trace("button: " + holder.button);
        }
    }
}
