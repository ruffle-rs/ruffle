package {

import flash.display.Sprite;
import flash.text.TextField;
import flash.ui.Mouse;
import flash.ui.MouseCursor;
import flash.events.KeyboardEvent;
import flash.ui.Keyboard;

public class Test extends Sprite {
	public function Test() {
		trace(Mouse.cursor);
		stage.addEventListener("keyDown", onKeyPress);

		var items = [null, "invalid", "AUTO", "ARROW", "BUTTON", "IBEAM", "HAND"];

		for each (var cursor:* in items) {
			try {
				Mouse.cursor = cursor;
				trace("Set cursor to " + Mouse.cursor);
			} catch(e) {
				trace(e.getStackTrace());
			}
		}

		var txt = new TextField();
		txt.width = 100;
		txt.height = 100;
		txt.x = 100;
		addChild(txt);

		var btn = new Sprite();
		btn.graphics.beginFill(0x0066CC); 
		btn.graphics.drawRect(0, 100, 100, 100); 
		btn.graphics.endFill();
		btn.buttonMode = true; 
		btn.useHandCursor = true;
		btn.addEventListener("click", onButtonClick);
		addChild(btn);
	
		var btn2 = new Sprite();
		btn2.graphics.beginFill(0x00CC66); 
		btn2.graphics.drawRect(100, 100, 100, 100); 
		btn2.graphics.endFill();
		btn2.buttonMode = true; 
		btn2.useHandCursor = true;
		btn2.addEventListener("mouseOver", onButtonOver);
		btn2.addEventListener("mouseOut", onButtonOut);
		addChild(btn2);
	}

	private function onButtonClick(e:*) {
		if (Mouse.cursor == MouseCursor.HAND) {
			Mouse.cursor = MouseCursor.AUTO;
		} else {
			Mouse.cursor = MouseCursor.HAND;
		}
	}

	private function onButtonOver(e:*) {
		Mouse.cursor = MouseCursor.ARROW;
	}

	private function onButtonOut(e:*) {
		Mouse.cursor = MouseCursor.AUTO;
	}

	public function onKeyPress(e:KeyboardEvent) {
		switch (e.keyCode) {
			case Keyboard.NUMBER_0:
				Mouse.cursor = "auto";
				break;
			case Keyboard.NUMBER_1:
				Mouse.cursor = "arrow";
				break;
			case Keyboard.NUMBER_2:
				Mouse.cursor = "button";
				break;
			case Keyboard.NUMBER_3:
				Mouse.cursor = "ibeam";
				break;
			case Keyboard.NUMBER_4:
				Mouse.cursor = "hand";
				break;
		}
	}
}

}
