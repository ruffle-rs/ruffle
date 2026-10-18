package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    public class Test extends Sprite {
        private var field:TextField = new TextField();
        private var fmt:TextFormat = new TextFormat();

        public function Test() {
            fmt.size = 30;

            trace("initial");
            reset();
            printSizes();

            test([]);

            var i:int;
            var j:int;
            for (i = -2; i <= 5; ++i) {
                test([i]);
            }
            for (i = -2; i <= 5; ++i) {
                for (j = -2; j <= 5; ++j) {
                    test([i, j]);
                }
            }

            test([1.7]);
            test(["2"]);
            test([3.9]);
            test([-0.5]);
            test([-1.5]);
            test(["-1"]);
            test([4294967295]);
            test([-2147483648]);
            test([2147483647]);
            test([undefined]);
            test([NaN]);

            test([1.7, 3.2]);
            test([0, 2147483647]);
            test([2, 4294967297]);
            test([-5, -5]);
            test([2147483647, -1]);
            test([0, -2147483648]);
            test([4294967295, 4294967295]);
            test([undefined, undefined]);
            test([NaN, NaN]);
            test([0, NaN]);

            test([], null);
            test([0, 1], null);
            test([4, 4], null);
            test([-2, -2], null);

            trace("empty text");
            field.text = "";
            test2([]);
            for (i = -2; i <= 1; ++i) {
                test2([i]);
            }
            for (i = -2; i <= 1; ++i) {
                for (j = -2; j <= 1; ++j) {
                    test2([i, j]);
                }
            }
            field.text = "ABCD";
            printSizes();
        }

        private function reset():void {
            field.htmlText = "<font size=\"12\">AB</font><font size=\"14\">CD</font>";
        }

        private function printSizes():void {
            var sizes:Array = [];
            for (var i:int = 0; i < field.length; ++i) {
                sizes.push(field.getTextFormat(i, i + 1).size);
            }
            trace("  text: " + field.text);
            trace("  sizes: " + sizes.join(","));
            trace("  default size: " + field.defaultTextFormat.size);
        }

        private function test(args:Array, ...rest):void {
            reset();
            test2(args, rest.length > 0 ? rest[0] : fmt);
        }

        private function test2(args:Array, f:TextFormat = null):void {
            if (arguments.length < 2) {
                f = fmt;
            }

            var label:String = "setTextFormat(" + (f == null ? "null" : "fmt");
            for (var i:int = 0; i < args.length; ++i) {
                label += ", " + String(args[i]);
            }
            trace(label + ")");

            try {
                if (args.length == 0) {
                    field.setTextFormat(f);
                } else if (args.length == 1) {
                    field.setTextFormat(f, args[0]);
                } else {
                    field.setTextFormat(f, args[0], args[1]);
                }
            } catch (e:Error) {
                trace("  " + Object(e).constructor);
                trace("  " + e.getStackTrace());
            }
            printSizes();
        }
    }
}
