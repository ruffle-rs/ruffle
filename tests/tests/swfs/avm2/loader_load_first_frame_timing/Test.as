package {
import flash.display.*;
import flash.events.*;
import flash.net.URLRequest;
import flash.system.fscommand;

// Traces only after a download completes: how many frames it takes differs between players.
public class Test extends MovieClip {
    private var framesLogged:int = 0;
    private var a:Loader = new Loader();
    private var b:Loader = new Loader();
    private var gotoInEnterFrame:Boolean = false;

    public function Test() {
        var self:Test = this;
        addFrameScript(
            0, function():void { Log.t("root frame 1 script"); stop(); },
            1, function():void { Log.t("root frame 2 script"); stop(); },
            2, function():void { Log.t("root frame 3 script"); stop(); },
            3, function():void { Log.t("root frame 4 script"); stop(); });

        // Registered first, so it runs before the logging listener below.
        stage.addEventListener(Event.ENTER_FRAME, function(e:Event):void {
            if (gotoInEnterFrame) {
                gotoInEnterFrame = false;
                Log.t("enterFrame handler: gotoAndStop(4)");
                self.gotoAndStop(4);
                Log.t("enterFrame handler: gotoAndStop(4) returned");
            }
        });
        stage.addEventListener(Event.ENTER_FRAME, function(e:Event):void { frameEvent("enterFrame"); });
        stage.addEventListener(Event.FRAME_CONSTRUCTED, function(e:Event):void { frameEvent("frameConstructed"); });
        stage.addEventListener(Event.EXIT_FRAME, function(e:Event):void { frameEvent("exitFrame"); });

        addChild(a);
        addChild(b);

        hook("a", a, function():void { self.play(); });
        hook("b", b, function():void {
            Log.t("progress handler: gotoAndStop(3)");
            self.gotoAndStop(3);
            Log.t("progress handler: gotoAndStop(3) returned");
            gotoInEnterFrame = true;
        });
        Log.loadeeName = "a";
        a.load(new URLRequest("loadee.swf"));
    }

    private function frameEvent(name:String):void {
        Log.t("stage " + name + " (root currentFrame=" + currentFrame + ", a.numChildren=" + a.numChildren + ", b.numChildren=" + b.numChildren + ")");
        if (name == "exitFrame" && Log.enabled && ++framesLogged == 3) {
            Log.enabled = false;
            framesLogged = 0;
            if (b.contentLoaderInfo.bytesTotal == 0) {
                Log.loadeeName = "b";
                b.load(new URLRequest("loadee.swf"));
            } else {
                fscommand("quit");
            }
        }
    }

    private function hook(n:String, l:Loader, onDownloaded:Function):void {
        var info:LoaderInfo = l.contentLoaderInfo;
        info.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
            if (e.bytesLoaded != e.bytesTotal) return;
            Log.enabled = true;
            Log.t("");
            Log.t(n + " progress " + e.bytesLoaded + "/" + e.bytesTotal);
            onDownloaded();
        });
        info.addEventListener(Event.INIT, function(e:Event):void { Log.t(n + " init: content=" + l.content); });
        info.addEventListener(Event.COMPLETE, function(e:Event):void { Log.t(n + " complete"); });
    }
}
}
