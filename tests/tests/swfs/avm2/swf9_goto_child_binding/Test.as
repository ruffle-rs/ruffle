package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        public static var constructions:int = 0;
        private var target:ProbeClip;
        private var sender:ProbeClip;
        private var ticks:int = 0;

        public function Test() {
            target = new ProbeClip();
            target.label = "target";
            addChild(target);
            sender = new ProbeClip();
            sender.label = "sender";
            addChild(sender);
            addEventListener("probe", onProbe);
            addFrameScript(0, run);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        public static function snapshot(label:String, clip:ProbeClip):void {
            trace(label + " frame=" + clip.currentFrame
                + " children=" + clip.numChildren
                + " slot=" + (clip.child != null)
                + " byName=" + (clip.getChildByName("child") != null)
                + " constructors=" + constructions);
        }

        private function run():void {
            stop();
            trace("BEGIN SWF9_GOTO_CHILD_BINDING");
            snapshot("before goto", target);
            target.gotoAndStop(2);
            snapshot("immediately after goto", target);
            trace("root frame script end");
        }

        private function onProbe(event:Event):void {
            snapshot("sibling event", target);
        }

        private function onEnterFrame(event:Event):void {
            ticks++;
            if (ticks == 3) {
                snapshot("final", target);
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                trace("SWF9_GOTO_CHILD_BINDING_COMPLETE");
            }
        }
    }
}
