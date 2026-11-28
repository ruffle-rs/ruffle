package {
import flash.display.*;
import flash.events.Event;
import flash.utils.ByteArray;
import flash.utils.getDefinitionByName;

public class Chain extends MovieClip {
    private static const Log:Object = getDefinitionByName("Log");
    private static const LoadeeBytes:Class = getDefinitionByName("LoadeeBytes") as Class;

    public function Chain() {
        Log.t("chain constructor");
        addFrameScript(0, function():void { Log.t("chain frame 1 script"); stop(); });
        addEventListener(Event.ADDED_TO_STAGE, function(e:Event):void {
            Log.t("chain addedToStage: loading inner");
            var loader:Loader = new Loader();
            DisplayObjectContainer(parent.parent).addChild(loader);
            loader.contentLoaderInfo.addEventListener(Event.INIT, function(e:Event):void { Log.t("inner init"); });
            Log.loadeeNames.push("inner");
            loader.loadBytes(new LoadeeBytes() as ByteArray);
            Log.t("inner loadBytes returned: numChildren=" + loader.numChildren);
        });
    }
}
}
