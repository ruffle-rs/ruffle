package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            var err:Error = new Error();

            Error.prototype.toString = function():String {
                trace("toString called (1)");
                return "from toString";
            };
            trace(err.getStackTrace());

            Error.prototype.toString = function():String {
                trace("toString called (2)");
                return null;
            };
            trace(err.getStackTrace());
        }
    }
}
