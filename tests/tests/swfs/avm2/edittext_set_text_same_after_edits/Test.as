package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    public class Test extends Sprite {
        private var field:TextField;
        private var big:TextFormat = new TextFormat();
        private var mid:TextFormat = new TextFormat();
        private var huge:TextFormat = new TextFormat();

        public function Test() {
            big.size = 30;
            mid.size = 20;
            huge.size = 40;

            var ops:Array = [
                ["appendText(\"\")", function():void { field.appendText(""); }],
                ["appendText(\"C\")", function():void { field.appendText("C"); }],
                ["replaceText(0, 1, \"A\")", function():void { field.replaceText(0, 1, "A"); }],
                ["replaceText(2, 2, \"\")", function():void { field.replaceText(2, 2, ""); }],
                ["replaceSelectedText(\"\")", function():void { field.setSelection(0, 0); field.replaceSelectedText(""); }],
                ["replaceSelectedText(\"X\")", function():void { field.setSelection(0, 1); field.replaceSelectedText("X"); }]
            ];

            for each (var op:Array in ops) {
                trace("// " + op[0] + ", then same text");
                setup();
                op[1]();
                print(op[0]);
                finish();

                trace("// defaultTextFormat, " + op[0] + ", then same text");
                setup();
                field.defaultTextFormat = mid;
                print("defaultTextFormat = mid");
                op[1]();
                print(op[0]);
                finish();

                trace("// " + op[0] + ", defaultTextFormat, then same text");
                setup();
                op[1]();
                print(op[0]);
                field.defaultTextFormat = mid;
                print("defaultTextFormat = mid");
                finish();
            }

            trace("// replaceText(0, 1, \"X\"), then old text");
            setup();
            field.replaceText(0, 1, "X");
            print("replaceText(0, 1, \"X\")");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// replaceSelectedText(\"X\"), then old text");
            setup();
            field.setSelection(0, 1);
            field.replaceSelectedText("X");
            print("replaceSelectedText(\"X\")");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// replaceSelectedText(\"X\"), then same text twice");
            setup();
            field.setSelection(0, 1);
            field.replaceSelectedText("X");
            print("replaceSelectedText(\"X\")");
            finish();
            finish();
        }

        private function setup():void {
            field = new TextField();
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
        }

        private function finish():void {
            field.setTextFormat(huge);
            print("setTextFormat(huge)");
            field.text = field.text;
            print("text = field.text");
        }

        private function print(label:String):void {
            var sizes:Array = [];
            for (var i:int = 0; i < field.length; ++i) {
                sizes.push(field.getTextFormat(i, i + 1).size);
            }
            trace("  " + label + ": text=" + field.text + " sizes=" + sizes.join(",")
                + " default=" + field.defaultTextFormat.size);
        }
    }
}
