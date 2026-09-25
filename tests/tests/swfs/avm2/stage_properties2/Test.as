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

        trace("Throwing properties:");
        this.logError(function() { stage.accessibilityImplementation = null; });
        this.logError(function() { stage.accessibilityProperties = null; });
        this.logError(function() { stage.alpha = 2; });
        this.logError(function() { stage.blendMode = null; });
        this.logError(function() { stage.cacheAsBitmap = true; });
        this.logError(function() { stage.contextMenu = null; });
        this.logError(function() { stage.filters = null; });
        this.logError(function() { stage.focusRect = null; });
        this.logError(function() { stage.height = 3; });
        this.logError(function() { stage.mask = null; });
        this.logError(function() { stage.mouseEnabled = true; });
        this.logError(function() { stage.name = null; });
        this.logError(function() { stage.opaqueBackground = null; });
        this.logError(function() { stage.rotation = 2; });
        this.logError(function() { stage.rotationX = 2; });
        this.logError(function() { stage.rotationY = 2; });
        this.logError(function() { stage.rotationZ = 2; });
        this.logError(function() { stage.scale9Grid = null; });
        this.logError(function() { stage.scaleX = 2; });
        this.logError(function() { stage.scaleY = 2; });
        this.logError(function() { stage.scaleZ = 2; });
        this.logError(function() { stage.scrollRect = 2; });
        this.logError(function() { stage.tabEnabled = true; });
        this.logError(function() { stage.tabIndex = 2; });
        this.logError(function() { stage.textSnapshot; });
        this.logError(function() { stage.transform = null; });
        this.logError(function() { stage.visible = true; });
        this.logError(function() { stage.width = 2; });
        this.logError(function() { stage.x = 2; });
        this.logError(function() { stage.y = 2; });
        this.logError(function() { stage.z = 2; });
        this.logError(function() { stage.colorCorrection = null; });
    }

    function logError(f:*):void {
        try {
            f();
        } catch(error) {
            trace("Error: " + error.getStackTrace());
        }
    }
}
}
