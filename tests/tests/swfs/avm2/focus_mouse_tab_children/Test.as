package {
import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.Shape;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.events.FocusEvent;
import flash.events.MouseEvent;
import flash.text.TextField;
import flash.text.TextFieldType;

// Clicking an object focuses it only when no ancestor has tabChildren = false.
[SWF(width="640", height="200", frameRate="30", backgroundColor="#000000")]
public class Test extends Sprite {
    public function Test() {
        // 0: a button directly on the root.
        addChild(newButton("rootButton", 0));

        // 1: a button whose parent has tabChildren = false.
        holder(1, false).addChild(newButton("buttonInNoTabParent", -1));

        // 2: a button whose grandparent has tabChildren = false.
        var middle:Sprite = new Sprite();
        middle.name = "middle";
        holder(2, false).addChild(middle);
        middle.addChild(newButton("buttonInNoTabGrandparent", -1));

        // 3: a buttonMode sprite whose parent has tabChildren = false.
        holder(3, false).addChild(newSprite("buttonModeSpriteInNoTabParent", -1));

        // 4: a button with tabEnabled explicitly true whose parent has tabChildren = false.
        var explicitButton:SimpleButton = newButton("tabEnabledButtonInNoTabParent", -1);
        explicitButton.tabEnabled = true;
        holder(4, false).addChild(explicitButton);

        // 5: an input text field whose parent has tabChildren = false.
        var text:TextField = new TextField();
        text.name = "inputTextInNoTabParent";
        text.type = TextFieldType.INPUT;
        text.border = true;
        text.borderColor = 0xFFFFFF;
        text.width = 70;
        text.height = 70;
        holder(5, false).addChild(text);

        // 6: a buttonMode sprite with tabChildren = false itself.
        var noTabSprite:Sprite = newSprite("buttonModeSpriteWithNoTabChildren", 6);
        noTabSprite.tabChildren = false;
        addChild(noTabSprite);

        // 7: a button whose parent had tabChildren = false, then true again.
        var restored:Sprite = holder(7, false);
        restored.tabChildren = true;
        restored.addChild(newButton("buttonInRestoredParent", -1));

        stage.addEventListener(MouseEvent.CLICK, onClick);
        stage.addEventListener(FocusEvent.MOUSE_FOCUS_CHANGE, onFocusEvent);
        stage.addEventListener(FocusEvent.FOCUS_IN, onFocusEvent);
        stage.addEventListener(FocusEvent.FOCUS_OUT, onFocusEvent);
    }

    private function holder(index:int, tabChildren:Boolean):Sprite {
        var s:Sprite = new Sprite();
        s.name = "holder" + index;
        s.x = index * 80;
        s.tabChildren = tabChildren;
        addChild(s);
        return s;
    }

    private function newButton(name:String, index:int):SimpleButton {
        var shape:Shape = new Shape();
        shape.graphics.beginFill(0x0000FF);
        shape.graphics.drawRect(0, 0, 70, 70);
        var button:SimpleButton = new SimpleButton(shape, shape, shape, shape);
        button.name = name;
        if (index >= 0) {
            button.x = index * 80;
        }
        return button;
    }

    private function newSprite(name:String, index:int):Sprite {
        var sprite:Sprite = new Sprite();
        sprite.name = name;
        sprite.graphics.beginFill(0x00FF00);
        sprite.graphics.drawRect(0, 0, 70, 70);
        sprite.buttonMode = true;
        if (index >= 0) {
            sprite.x = index * 80;
        }
        return sprite;
    }

    private function describe(obj:Object):String {
        if (obj == null) {
            return "null";
        }
        if (obj == stage) {
            return "stage";
        }
        return obj.name;
    }

    private function onClick(e:MouseEvent):void {
        trace("click " + describe(e.target) + ", focus: " + describe(stage.focus));
    }

    private function onFocusEvent(e:FocusEvent):void {
        trace("  " + e.type + " on " + describe(e.target) + ", relatedObject: " + describe(e.relatedObject) + ", focus: " + describe(stage.focus));
    }
}
}
