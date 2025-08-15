package {
import flash.display.Sprite;
import flash.display.Stage;

public class Test extends Sprite {
    function Test() {
        super();
        var stage:Stage = this.stage;
        trace("===== height");
        this.logError(function() {
            trace(stage.height);
        });
        this.logError(function() {
            trace(stage.height = 400);
        });
        trace("===== width");
        this.logError(function() {
            trace(stage.width);
        });
        this.logError(function() {
            trace(stage.width = 400);
        });
        trace("===== textSnapshot");
        this.logError(function() {
            trace(stage.textSnapshot);
        });
    }

    function logError(f:*):void {
        try {
            f();
        } catch(error) {
            trace("Error: " + error);
        }
    }
}
}
