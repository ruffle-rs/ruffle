package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    public class Test extends Sprite {
        private var field:TextField;

        public function Test() {
            var big:TextFormat = new TextFormat();
            big.size = 30;
            var mid:TextFormat = new TextFormat();
            mid.size = 20;
            var twelve:TextFormat = new TextFormat();
            twelve.size = 12;

            trace("// setTextFormat, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.text = "AB";
            print("text = \"AB\"");
            field.text = "ab";
            print("text = \"ab\"");

            trace("// htmlText, then same text");
            fresh();
            field.htmlText = "<font size=\"30\">AB</font>";
            print("htmlText = \"<font size='30'>AB</font>\"");
            field.text = "AB";
            print("text = \"AB\"");
            field.text = "ab";
            print("text = \"ab\"");

            trace("// defaultTextFormat, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.defaultTextFormat = big;
            print("defaultTextFormat = big");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// setTextFormat and defaultTextFormat, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// htmlText and defaultTextFormat, then same text");
            fresh();
            field.htmlText = "<font size=\"30\">AB</font>";
            print("htmlText = \"<font size='30'>AB</font>\"");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// defaultTextFormat set to its own value, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.defaultTextFormat = field.defaultTextFormat;
            print("defaultTextFormat = field.defaultTextFormat");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// defaultTextFormat set to an equal value, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.defaultTextFormat = twelve;
            print("defaultTextFormat = twelve");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// defaultTextFormat changed and changed back, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.defaultTextFormat = twelve;
            print("defaultTextFormat = twelve");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// defaultTextFormat, setTextFormat, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// defaultTextFormat, same text twice");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.text = "AB";
            print("text = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// defaultTextFormat, different text, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.text = "XY";
            print("text = \"XY\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.text = "XY";
            print("text = \"XY\"");

            trace("// defaultTextFormat, htmlText, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            field.htmlText = "AB";
            print("htmlText = \"AB\"");
            field.setTextFormat(big);
            print("setTextFormat(big)");
            field.text = "AB";
            print("text = \"AB\"");

            trace("// textColor, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            printColors();
            field.setTextFormat(big);
            print("setTextFormat(big)");
            printColors();
            field.textColor = 0xFF0000;
            print("textColor = 0xFF0000");
            printColors();
            field.text = "AB";
            print("text = \"AB\"");
            printColors();

            trace("// defaultTextFormat, textColor, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            printColors();
            field.setTextFormat(big);
            print("setTextFormat(big)");
            printColors();
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            printColors();
            field.textColor = 0xFF0000;
            print("textColor = 0xFF0000");
            printColors();
            field.text = "AB";
            print("text = \"AB\"");
            printColors();

            trace("// textColor, defaultTextFormat, then same text");
            fresh();
            field.text = "AB";
            print("text = \"AB\"");
            printColors();
            field.setTextFormat(big);
            print("setTextFormat(big)");
            printColors();
            field.textColor = 0xFF0000;
            print("textColor = 0xFF0000");
            printColors();
            field.defaultTextFormat = mid;
            print("defaultTextFormat = mid");
            printColors();
            field.text = "AB";
            print("text = \"AB\"");
            printColors();

            var html:String;
            trace("// same htmlText");
            prepareHtml();
            field.htmlText = field.htmlText;
            printHtml("htmlText = field.htmlText");

            trace("// defaultTextFormat (own value), same htmlText");
            prepareHtml();
            field.defaultTextFormat = field.defaultTextFormat;
            printHtml("defaultTextFormat = field.defaultTextFormat");
            field.htmlText = field.htmlText;
            printHtml("htmlText = field.htmlText");

            trace("// defaultTextFormat (size 20), same htmlText");
            prepareHtml();
            html = field.htmlText;
            field.defaultTextFormat = mid;
            printHtml("defaultTextFormat = mid");
            field.htmlText = html;
            printHtml("htmlText = html");

            trace("// defaultTextFormat, same htmlText, same htmlText again");
            prepareHtml();
            field.defaultTextFormat = field.defaultTextFormat;
            printHtml("defaultTextFormat = field.defaultTextFormat");
            field.htmlText = field.htmlText;
            printHtml("htmlText = field.htmlText");
            field.replaceText(1, 2, "");
            printHtml("replaceText(1, 2, \"\")");
            field.htmlText = field.htmlText;
            printHtml("htmlText = field.htmlText");

            trace("// defaultTextFormat, same htmlText, then same text");
            prepareHtml();
            field.defaultTextFormat = field.defaultTextFormat;
            printHtml("defaultTextFormat = field.defaultTextFormat");
            field.htmlText = field.htmlText;
            printHtml("htmlText = field.htmlText");
            field.setTextFormat(big);
            printHtml("setTextFormat(big)");
            field.text = field.text;
            printHtml("text = field.text");
        }

        private function fresh():void {
            field = new TextField();
        }

        private function prepareHtml():void {
            field = new TextField();
            field.multiline = true;
            field.htmlText = "<p align=\"center\">x</p>";
            field.replaceText(1, 2, "");
            printHtml("prepared");
        }

        private function printHtml(label:String):void {
            trace("  " + label + ": length=" + field.length + " size=" + field.getTextFormat(0, 1).size
                + " default=" + field.defaultTextFormat.size);
        }

        private function printColors():void {
            trace("    colors=" + field.getTextFormat(0, 1).color + "," + field.getTextFormat(1, 2).color
                + " default color=" + field.defaultTextFormat.color);
        }

        private function print(label:String):void {
            trace("  " + label + ": text=" + field.text + " sizes=" + field.getTextFormat(0, 1).size
                + "," + field.getTextFormat(1, 2).size + " default=" + field.defaultTextFormat.size);
        }
    }
}
