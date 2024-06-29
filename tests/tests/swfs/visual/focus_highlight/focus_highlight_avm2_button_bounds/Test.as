package {

import flash.display.MovieClip;
import flash.display.Shape;
import flash.display.SimpleButton;
import flash.events.FocusEvent;

[SWF(width="50", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    private var button: SimpleButton;

    public function Test() {
        var buttonState = new Shape();
        buttonState.graphics.beginFill(0xFF00FF);
        buttonState.graphics.drawRect(-10, -10, 20, 20);
        buttonState.graphics.endFill();
        var hitState = new Shape();
        hitState.graphics.beginFill(0xFF0000);
        hitState.graphics.drawRect(-5, -5, 10, 10);
        hitState.graphics.endFill();

        button = new SimpleButton();
        button.hitTestState = hitState;
        button.upState = buttonState;
        button.downState = buttonState;
        button.overState = buttonState;
        button.x = 25;
        button.y = 25;
        button.tabIndex = 1;

        stage.addChild(button);

        stage.addEventListener("focusIn", function (evt: FocusEvent): void {
            trace("Focus changed: " + evt.relatedObject + " -> " + evt.target);
        });
    }
}
}
