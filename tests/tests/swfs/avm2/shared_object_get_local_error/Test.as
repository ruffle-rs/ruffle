package {
    import flash.display.Sprite;
    import flash.net.SharedObject;

    // What SharedObject.getLocal does when it can't create a shared object.
    public class Test extends Sprite {
        public function Test() {
            test("valid", "so", null, false);
            test("invalid name", "a:b", null, false);
            test("empty localPath", "so", "", false);
            test("unrelated localPath", "so", "/nonexistent", false);
            test("secure", "so", null, true);
        }

        private function test(label:String, name:String, localPath:String, secure:Boolean):void {
            try {
                var so:SharedObject = SharedObject.getLocal(name, localPath, secure);
                trace(label + ": " + (so == null ? "null" : "object"));
            } catch (e:Error) {
                trace(label + ": " + Object(e).constructor);
                trace(e.getStackTrace());
            }
        }
    }
}
