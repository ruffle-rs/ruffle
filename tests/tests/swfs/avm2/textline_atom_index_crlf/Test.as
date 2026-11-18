package {
    import flash.display.Sprite;
    import flash.text.engine.ElementFormat;
    import flash.text.engine.FontDescription;
    import flash.text.engine.TextBlock;
    import flash.text.engine.TextElement;
    import flash.text.engine.TextLine;

    public class Test extends Sprite {
        public function Test() {
            for each (var text:String in ["ab\r\ncd", "ab\ncd", "ab\rcd", "ab cd"]) {
                var format:ElementFormat = new ElementFormat(new FontDescription("_sans"), 12);
                var block:TextBlock = new TextBlock(new TextElement(text, format));
                var line:TextLine = block.createTextLine(null, 1000);
                var n:int = 0;

                while (line != null) {
                    var out:String = "text " + escape(text) + " line " + n
                        + ": begin " + line.textBlockBeginIndex
                        + ", rawTextLength " + line.rawTextLength
                        + ", atomCount " + line.atomCount + " |";
                    var end:int = line.textBlockBeginIndex + line.rawTextLength;

                    for (var c:int = line.textBlockBeginIndex; c < end; c++) {
                        var atom:int = line.getAtomIndexAtCharIndex(c);
                        var result:String;

                        try {
                            line.getAtomBounds(atom);
                            result = "ok";
                        } catch (e:Error) {
                            result = "#" + e.errorID;
                        }

                        out += " c" + c + "->a" + atom + ":" + result;
                    }

                    trace(out);
                    line = block.createTextLine(line, 1000);
                    n++;
                }
            }
        }
    }
}
