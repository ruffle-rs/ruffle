package {
import flash.display.*;
import flash.events.*;
import flash.system.fscommand;
import flash.utils.ByteArray;
import flash.utils.Timer;

public class Test extends Sprite {
    public function Test() {
        var self:Test = this;
        stage.addEventListener(Event.ENTER_FRAME, function(e:Event):void {
            Log.frame++;
            Log.t("stage enterFrame");
            if (Log.frame == 2) {
                load("b", new LoadeeBytes());
            } else if (Log.frame == 4) {
                load("chain", new ChainBytes());
            } else if (Log.frame == 7) {
                var timer:Timer = new Timer(1, 1);
                timer.addEventListener(TimerEvent.TIMER, function(te:TimerEvent):void {
                    Log.t("timer");
                    self.load("d", new LoadeeBytes());
                });
                timer.start();
            } else if (Log.frame == 10) {
                fscommand("quit");
            }
        });
        stage.addEventListener(Event.FRAME_CONSTRUCTED, function(e:Event):void { Log.t("stage frameConstructed"); });
        stage.addEventListener(Event.EXIT_FRAME, function(e:Event):void { Log.t("stage exitFrame"); });
        load("a", new LoadeeBytes());
    }

    private function load(name:String, bytes:ByteArray):void {
        var loader:Loader = new Loader();
        addChild(loader);
        loader.contentLoaderInfo.addEventListener(Event.INIT, function(e:Event):void { Log.t(name + " init"); });
        loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void { Log.t(name + " complete"); });
        if (name != "chain") Log.loadeeNames.push(name);
        loader.loadBytes(bytes);
        Log.t(name + " loadBytes returned: numChildren=" + loader.numChildren);
    }
}
}
