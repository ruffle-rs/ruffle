package {
import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends MovieClip {
    private var obj1: TextField;
    private var obj2: TextField;
    private var obj3: TextField;

    public function Test() {
        stage.scaleMode = "noScale";

        obj1 = new TextField();
        obj1.type = "input";
        obj1.border = true;
        obj1.name = "obj1";
        obj1.x = 70;
        obj1.y = 10;
        obj1.width = 10;
        obj1.height = 10;

        obj2 = new TextField();
        obj2.type = "input";
        obj2.border = true;
        obj2.name = "obj2";
        obj2.x = 10;
        obj2.y = 20;
        obj2.width = 10;
        obj2.height = 10;

        obj3 = new TextField();
        obj3.type = "input";
        obj3.border = true;
        obj3.name = "obj3";
        obj3.x = 40;
        obj3.y = 40;
        obj3.width = 10;
        obj3.height = 10;

        stage.focus = obj1;

        for each (var obj in [obj1, obj2, obj3]) {
            obj.addEventListener("focusIn", function (evt:FocusEvent):void {
                trace("Focus changed: " + evt.relatedObject.name + " -> " + evt.target.name);
            });
            this.stage.addChild(obj);
        }
    }
}
}
