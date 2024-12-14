package {
    import flash.display.Sprite;
    import flash.text.StyleSheet;
    import flash.text.TextField;
    import flash.text.TextFormat;
    import flash.text.engine.FontDescription;
    import flash.text.engine.ElementFormat;

    public class Test extends Sprite {
        public function Test() {
            var font:String = "Tinos"

            var tf1:TextField = createCustomTextField(20, 20, 500, 80);
            tf1.multiline = true;
            tf1.htmlText = "Abc123 <i>Abc</i> <b>XyZ</b><br>Hello WWWWWWWW!";

            var myTextFormat1:TextFormat = new TextFormat();
            myTextFormat1.font = font;
            myTextFormat1.size = 30;
            tf1.setTextFormat(myTextFormat1)
        }

        private function createCustomTextField(x:Number, y:Number, width:Number, height:Number):TextField {
            var result:TextField = new TextField();
            result.x = x;
            result.y = y;
            result.width = width;
            result.height = height;
            addChild(result);
            return result;
        }
    }
}
