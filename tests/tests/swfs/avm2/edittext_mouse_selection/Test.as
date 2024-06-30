package {
import flash.display.Sprite;
import flash.events.KeyboardEvent;
import flash.events.MouseEvent;
import flash.events.TextEvent;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.ui.Keyboard;

[SWF(width="400", height="200")]
public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007E")]
    private var notoSans:Class;

    private var text:TextField;

    public function Test() {
        text = new TextField();
        text.border = true;
        text.width = 400;
        text.height = 200;
        text.type = "input";
        text.wordWrap = true;
        text.multiline = true;
        var tf = new TextFormat();
        tf.font = "Noto Sans";
        tf.size = 24;
        text.defaultTextFormat = tf;
        addChild(text);

        stage.focus = text;
        stage.addEventListener(TextEvent.TEXT_INPUT, textInput);
        stage.addEventListener(KeyboardEvent.KEY_DOWN, keyPressedDown);
        stage.addEventListener(MouseEvent.MOUSE_DOWN, mouseDown);
        stage.addEventListener(MouseEvent.MOUSE_UP, mouseUp);
        stage.addEventListener(MouseEvent.MOUSE_MOVE, mouseMove);
    }

    private function keyPressedDown(event:KeyboardEvent):void {
        if (event.keyCode == Keyboard.ESCAPE) {
            trace("Selected: " + text.selectedText.replace(/\r/g, "\\n").replace(/\n/g, "\\n"));
            text.setSelection(0, 0);
        } else if (event.keyCode == Keyboard.NUMBER_1) {
            text.text = "word1_word2_word3";
        } else if (event.keyCode == Keyboard.NUMBER_2) {
            text.text = "word1 word2 word3";
        } else if (event.keyCode == Keyboard.NUMBER_3) {
            text.text = "word1 word2 word3 word4 word5 word6 word7 word8 word9\nword10 word11 word12";
        } else if (event.keyCode == Keyboard.NUMBER_4) {
            text.selectable = false;
        } else if (event.keyCode == Keyboard.NUMBER_5) {
            text.selectable = true;
        } else if (event.keyCode == Keyboard.NUMBER_6) {
            text.text = "word1     word2";
        }
        trace("{ \"type\": \"KeyDown\", \"key_code\": " + event.keyCode + " },")
    }

    private function mouseDown(event:MouseEvent):void {
        trace("{ \"type\": \"MouseDown\", \"pos\": [" + event.stageX + ", " + event.stageY + "], \"btn\": \"Left\" },");
    }

    private function mouseUp(event:MouseEvent):void {
        trace("{ \"type\": \"MouseUp\", \"pos\": [" + event.stageX + ", " + event.stageY + "], \"btn\": \"Left\" },");
    }

    private function mouseMove(event:MouseEvent):void {
        trace("{ \"type\": \"MouseMove\", \"pos\": [" + event.stageX + ", " + event.stageY + "] },");
    }

    private function textInput(event:TextEvent):void {
        if (event.text.search(/[0-9]/g) != -1) {
            event.preventDefault();
        }
    }
}
}
