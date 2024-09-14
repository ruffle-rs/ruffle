// https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html#restrict
// See the analogous test from avm1 for description.

package {
    import flash.display.Sprite;
    import flash.display.Stage;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.ui.Keyboard;
    import flash.text.TextField;

    public class Test extends Sprite {
        private var player:Sprite;
        private var text:TextField;

        private var currentRestrict:int = -1;
        private var restricts:Array = [
            // different empty values
            undefined,
            null,
            "",
            false,
            // non-empty non-string values
            1,
            true,
            0.1,
            NaN,
            new Object(),
            // only selected chars
            "aB*Δ",
            "aa",
            // ASCII ranges
            "a-z",
            "A-Z",
            "a-bA",
            // non-standard ranges
            "a-",
            "-b",
            "b-a",
            "A-z",
            "-",
            "--",
            "---",
            "----",
            "-----",
            "-----b",
            "b-----",
            "a-b-c",
            "a-b-A",
            "a-a-b",
            "\\\\-\\^",
            "\\^-\\\\",
            // various behaviors with caret ^
            "^",
            "^^",
            "\\^a",
            "^\\^",
            "^aą",
            "a^b^c",
            "a^b^c^A^B",
            "a-zA-Z^bC",
            "a-zA-Z^",
            // escapes
            "\\-",
            "a\\-z",
            "\\\\",
            "\\^",
            "\\ab",
            "a\\",
            "\u0020-\u007E",
            // unicode range
            "α-ω"
        ];

        public function Test() {
            text = new TextField();
            text.border = true;
            text.width = 200;
            text.height = 20;
            text.type = "input";
            addChild(text);

            stage.focus = text;
            stage.addEventListener(KeyboardEvent.KEY_DOWN, keyPressedDown);
        }

        private function keyPressedDown(event:KeyboardEvent):void {
            if (event.keyCode == 27) {
                nextRestrict();
            }
        }

        private function nextRestrict():void {
            trace("Text: '" + text.text + "'");
            trace("====================");
            text.text = "";
            currentRestrict += 1;
            if (restricts.length <= currentRestrict) {
                trace("No more restricts");
                return;
            }
            text.restrict = restricts[currentRestrict];
            trace("Restrict set: '" + restricts[currentRestrict] + "'");
            trace("Restrict get: '" + text.restrict + "'");
        }
    }
}
