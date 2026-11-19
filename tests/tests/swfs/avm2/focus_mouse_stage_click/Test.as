package {
import flash.display.InteractiveObject;
import flash.display.Shape;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.events.FocusEvent;
import flash.events.MouseEvent;

// Clicking the stage clears the focus, even when the focused object
// could not have been focused by mouse.
[SWF(width="640", height="200", frameRate="30", backgroundColor="#000000")]
public class Test extends Sprite {
    private var targets:Array = [];
    private var step:int = 0;

    public function Test() {
        var rootButton:SimpleButton = newButton("rootButton");
        rootButton.x = 0;
        addChild(rootButton);

        var holder:Sprite = new Sprite();
        holder.x = 80;
        holder.tabChildren = false;
        addChild(holder);
        var noTabButton:SimpleButton = newButton("buttonInNoTabParent");
        holder.addChild(noTabButton);

        var disabledButton:SimpleButton = newButton("tabDisabledButton");
        disabledButton.tabEnabled = false;
        disabledButton.x = 160;
        addChild(disabledButton);

        targets = [rootButton, noTabButton, disabledButton];
        stage.addEventListener(MouseEvent.CLICK, onClick);
        stage.addEventListener(FocusEvent.MOUSE_FOCUS_CHANGE, onFocusEvent);
        stage.addEventListener(FocusEvent.FOCUS_IN, onFocusEvent);
        stage.addEventListener(FocusEvent.FOCUS_OUT, onFocusEvent);
        next();
    }

    private function newButton(name:String):SimpleButton {
        var shape:Shape = new Shape();
        shape.graphics.beginFill(0x0000FF);
        shape.graphics.drawRect(0, 0, 70, 70);
        var button:SimpleButton = new SimpleButton(shape, shape, shape, shape);
        button.name = name;
        return button;
    }

    private function describe(obj:InteractiveObject):String {
        return obj ? obj.name : "null";
    }

    private function next():void {
        if (step < targets.length) {
            stage.focus = targets[step];
            trace("focus set to " + describe(stage.focus));
        }
        step++;
    }

    private function onFocusEvent(e:FocusEvent):void {
        trace("  " + e.type + " on " + (e.target == stage ? "stage" : e.target.name) + ", relatedObject: " + describe(e.relatedObject) + ", focus: " + describe(stage.focus));
    }

    private function onClick(e:MouseEvent):void {
        trace("click on " + (e.target == stage ? "stage" : e.target.name) + ", focus: " + describe(stage.focus));
        next();
    }
}
}
