package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    public class Test extends Sprite {
        private var field:TextField = new TextField();

        public function Test() {
            field.htmlText = "<b><font size=\"10\" color=\"#FF0000\">A</font></b>"
                + "<font size=\"12\" color=\"#FF0000\">B</font>"
                + "<font size=\"14\" color=\"#0000FF\">C</font>"
                + "<i><font size=\"16\" color=\"#0000FF\">D</font></i>";
            testAll(-2, 5);

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

            trace("empty text");
            field.text = "";
            testAll(-2, 1);
        }

        private function testAll(min:int, max:int):void {
            var i:int;
            var j:int;
            test([]);
            for (i = min; i <= max; ++i) {
                test([i]);
            }
            for (i = min; i <= max; ++i) {
                for (j = min; j <= max; ++j) {
                    test([i, j]);
                }
            }
        }

        private function test(args:Array):void {
            var label:String = "getTextFormat(";
            for (var i:int = 0; i < args.length; ++i) {
                label += (i > 0 ? ", " : "") + String(args[i]);
            }
            trace(label + ")");

            try {
                var tf:TextFormat;
                if (args.length == 0) {
                    tf = field.getTextFormat();
                } else if (args.length == 1) {
                    tf = field.getTextFormat(args[0]);
                } else {
                    tf = field.getTextFormat(args[0], args[1]);
                }
                trace("  size=" + tf.size + " color=" + tf.color + " bold=" + tf.bold
                    + " italic=" + tf.italic + " underline=" + tf.underline);
            } catch (e:Error) {
                trace("  " + Object(e).constructor);
                trace("  " + e.getStackTrace());
            }
        }
    }
}
