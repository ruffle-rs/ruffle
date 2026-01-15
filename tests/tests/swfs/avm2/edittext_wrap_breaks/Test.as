package {

import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends Sprite {
    [Embed(source="TestWrapBreaks.ttf", fontName="TestFont", embedAsCFF="false")]
    private var TestFont:Class;

    public function Test() {
        var charsText = [
            "a", // ASCII
            "ź", // Latin non-ASCII
            "形", // Chinese
            "글", // Korean
            "の", // Japanese
            "￥", "１", "ａ", // Full width
        ];
        var charsAscii = [
            "a", " ", "!", "@", "#", "$", "%", "^",
            "&", "*", "(", ")", "_", "+", "-", "=",
            "[", "]", "{", "}", "`", "|", ";", ":",
            "'", "~", "/", "?", ".", ">", ",", "<",
            "\"", "\\"
        ];
        var charsNonAsciiLatin = [
            "¬", "≠", "²", "³", "¢", "€", "½", "§",
            "·", "«", "»", "–", ".", "≥", "∨", "¡",
            "¿", "£", "¼", "‰", "∧", "≈", "¾", "±",
            "°", "—", "÷", "ą"
        ];
        var charsFullwidthPunctuation = [
            "◦", "♪", "_", "＿", "﹏", "－", "—", "⸺",
            "〜", "゠", ",", "，", "、", "；", "：", "！",
            "？", "．", "‥", "…", "。", "·", "＇", "＂",
            "“", "”", "〝", "〟", "（", "）", "［", "］",
            "｛", "｝", "｟", "｠", "⟨", "⟩", "〈", "〉",
            "《", "》", "「", "﹁", "」", "﹂", "『", "﹃",
            "』", "﹄", "【", "】", "＠", "*", "＊", "／",
            "＼", "＆", "＃", "％", "•", "〽", "｀", "＾",
            "￣", "＋", "<", "＜", "＝", ">", "＞", "￢",
            "｜", "￤", "～", "≪", "≫", "□", "▯", "○"
        ];
        var charsTextReduced = [
            "a", "形", "글", "の"
        ];
        var charsBrackets = [
            "⁅", "⁆", "⎰", "⎱", "⎴", "⎵", "❬", "❭",
            "❰", "❱", "❲", "❳", "❴", "⟩", "⟪", "⟭",
            "⦃", "⦈", "⦉", "⦊", "⦋", "⦒", "⦓", "⦔",
            "⦕", "⦘", "⧼", "⸊", "⸌", "⸍", "⸜", "⸝",
            "⸢", "⸣", "⸤", "⸥", "⸦", "〉", "《", "」",
            "『", "』", "【", "〕", "〖", "〙", "〚", "︺",
            "︻", "﹀", "﹁", "﹂", "﹃", "﹄", "﹇", "﹞",
            //"［", "｣", "𝄕", "⁽", "⁾", "₍", "₎",
            "⎛", "⎜", "⎝", "⎞", "⎟", "⎠", "⏜", "⏝",
            "❨", "❩", "❪", "❫", "⟮", "⟯", "⦅", "⦆",
            "⸨", "⸩", "﴾", "﴿", "︵", "︶", "﹙", "﹚",
            "（", "）", "｟", "｠", "⟦", "⟧", "⦍",
            "⦎", "⦏", "⦐", "⸧"
        ];

        // ASCII vs ASCII
        for each (var ch1 in charsAscii) {
            for each (var ch2 in charsAscii) {
                testBreak(ch1, ch2);
            }
        }

        // Text vs text
        for each (var ch1 in charsText) {
            for each (var ch2 in charsText) {
                testBreak(ch1, ch2);
            }
        }

        // Text vs ASCII
        for each (var ch1 in charsAscii) {
            for each (var ch2 in charsText) {
                testBreak(ch1, ch2);
                testBreak(ch2, ch1);
            }
        }

        // Non-ASCII with text
        for each (var chA in charsNonAsciiLatin) {
            for each (var chB in charsText) {
                testBreak(chA, chB);
                testBreak(chB, chA);
            }
        }

        // Full-width punctuation vs text
        for each (var chA in charsFullwidthPunctuation) {
            for each (var chB in charsText) {
                testBreak(chA, chB);
                testBreak(chB, chA);
            }
        }

        // Brackets
        for each (var chA in charsBrackets) {
            for each (var chB in charsTextReduced) {
                testBreak(chA, chB);
                testBreak(chB, chA);
            }
        }

        trace("Done");
    }

    private function testBreak(left:String, right:String):void {
        if (breaksBetween(left, right)) {
            trace(left + right + ": breaks");
        }
    }

    private function breaksBetween(left:String, right:String):Boolean {
        if (left.length != 1) {
            throw new Error(left.length + ": " + left);
        }
        if (right.length != 1) {
            throw new Error(right.length + ": " + right);
        }

        var tf:TextField = new TextField();
        tf.height = 100;
        // Make it super wide, will change the width later.
        tf.width = 1000;
        tf.embedFonts = true;
        tf.defaultTextFormat = new TextFormat("TestFont", 10);
        tf.multiline = true;
        tf.wordWrap = true;
        tf.border = true;

        // Now prepend a string that has a soft break, so that we can observe
        // breaking when left and right should not break.
        tf.text = "a a" + left + right;

        if (tf.getLineLength(0) != 5) {
            // Field is wide enough that it should contain all the text.
            throw new Error();
        }

        // Make sure we're not missing any glyphs.
        for (var i = 0; i < tf.getLineLength(0); ++i) {
            var b = tf.getCharBoundaries(i);
            if (b == null || b.width < 1) {
                throw Error("Missing glyph for " + tf.text.charAt(i));
            }
        }

        tf.width = Math.ceil(tf.getLineMetrics(0).width) + 5;
        // Force relayout
        tf.text = tf.text;

        if (tf.getLineLength(0) != 5) {
            throw new Error();
        }

        for (var i:int = 0; i < 100; i += 1) {
            tf.width -= 1;

            // Force relayout
            tf.text = tf.text;

            var len:int = tf.getLineLength(0);
            if (len >= 5) {
                continue;
            }

            if (len == 4) {
                // The text broke between left and right.
                return true;
            } else {
                // The text broke somewhere else.
                return false;
            }
        }

        throw new Error();
    }
}

}
