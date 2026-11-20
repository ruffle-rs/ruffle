package {
    import flash.display.Sprite;
    import flash.net.SharedObject;

    public class Test extends Sprite {
        public function Test() {
            var names:Array = [
                "save", ".save", "..save", "...", "dir/.save", "x/y/z",
                "", "/", "/save", "save/",
                ".", "..", "./save", "././save", "./a/b", "a/./b", "a/b/./c", "./.", "./..", "save/.", "save/..", "a/./.",
                "../save", "dir/../save", "a/b/../c", "a/./../b", "a//b"
            ];
            for each (var name:String in names) {
                trace("\"" + name + "\": " + describe(name));
            }

            same("save", "./save");
            same("save", "././save");
            same("a/b", "./a/b");
            same("a/b", "a/./b");
            same("a/b/c", "a/b/./c");
            same(".", "./.");
            same("..", "./..");
            same("save", "/save");
            same("save", "save/");
            same(".save", "save");
        }

        private function describe(name:String):String {
            try {
                return SharedObject.getLocal(name) != null ? "ok" : "null";
            } catch (e:Error) {
                return "Error #" + e.errorID;
            }
            return "";
        }

        private function same(a:String, b:String):void {
            trace("\"" + a + "\" == \"" + b + "\": " + (SharedObject.getLocal(a) == SharedObject.getLocal(b)));
        }
    }
}
