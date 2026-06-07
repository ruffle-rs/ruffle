package {
import flash.display.Sprite;
import flash.display.StageAlign;
import flash.display.StageScaleMode;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Minimal reproduction of the "Send E-Mail" description box from Multics.fla.
//
// The .fla authors that box as a fixed TLF container:
//   <DOMTLFText right="2940" bottom="1320">          -> 147 x 66 px
//   <tlfTextObject paddingLeft/Top/Right/Bottom="2"   -> text width 143 px
//                  verticalAlign="top">
//   <span fontFamily="_sans" fontSize="12" kerning="off"
//         lineHeight="120%" trackingRight="-3%">
//
// This SWF draws that box exactly as TLF composes it (ROMAN_UP leading,
// centered lines, 12 px _sans) so the rendering can be diffed against
// Flash Player, and also traces the line breaks for a metric check.
public class Test extends Sprite {
    private static const TXT:String =
        "Send an e-mail using an authentic simulation of the CTSS e-mail program";
    private static const BOX_W:Number = 147;
    private static const BOX_H:Number = 66;
    private static const PAD:Number = 2;
    private static const SIZE:Number = 12;
    private static const LINE_HEIGHT:Number = SIZE * 1.2;   // lineHeight 120%
    private static const TEXT_W:Number = BOX_W - 2 * PAD;   // 143

    // TLF resolves trackingRight="-3%" to -3% of the 12 px size.
    private static const TRACKING:Number = -0.36;

    public function Test() {
        if (stage) {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;
        }

        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var authored:Sprite = renderBox(TRACKING);
        authored.scaleX = authored.scaleY = 3;
        authored.x = 28;
        authored.y = 70;
        addChild(authored);

        var noTracking:Sprite = renderBox(0);
        noTracking.scaleX = noTracking.scaleY = 3;
        noTracking.x = 28;
        noTracking.y = 70 + BOX_H * 3 + 24;
        addChild(noTracking);

        wrap("no tracking", 0);
        wrap("authored -3%", TRACKING);
    }

    private function makeFormat(tracking:Number):ElementFormat {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, SIZE);
        ef.color = 0x000000;
        ef.kerning = "off";
        ef.trackingRight = tracking;
        return ef;
    }

    private function renderBox(tracking:Number):Sprite {
        var box:Sprite = new Sprite();
        box.graphics.lineStyle(1, 0x000000);
        box.graphics.beginFill(0xFFFFFF);
        box.graphics.drawRect(0, 0, BOX_W, BOX_H);
        box.graphics.endFill();

        var tb:TextBlock = new TextBlock(new TextElement(TXT, makeFormat(tracking)));
        var line:TextLine = tb.createTextLine(null, TEXT_W);
        var i:int = 0;
        while (line != null) {
            // TLF ROMAN_UP leading, verticalAlign top:
            //   first baseline = PAD + ascent, each next + lineHeight.
            line.y = PAD + line.ascent + i * LINE_HEIGHT;
            // textAlign center.
            line.x = PAD + (TEXT_W - line.textWidth) / 2;
            box.addChild(line);
            line = tb.createTextLine(line, TEXT_W);
            i++;
        }
        return box;
    }

    private function wrap(label:String, tracking:Number):void {
        var tb:TextBlock = new TextBlock(new TextElement(TXT, makeFormat(tracking)));
        var line:TextLine = tb.createTextLine(null, TEXT_W);
        var n:int = 0;
        trace("--- " + label + " (trackingRight=" + tracking + ") ---");
        while (line != null) {
            var s:int = line.textBlockBeginIndex;
            var e:int = s + line.rawTextLength;
            trace("  line " + n + " [" + s + "," + e + ")"
                + " width=" + int(line.textWidth + 0.5)
                + " \"" + TXT.substring(s, e) + "\"");
            line = tb.createTextLine(line, TEXT_W);
            n++;
        }
        trace("  lineCount=" + n);
    }
}
}
