package {
    import flash.display.MovieClip;
    import flash.events.Event;
    public class Test extends MovieClip {
        public var clip:MovieClip;
        private var objs:Array = [];
        private var ticks:int = 0;
        public function Test() {
            addFrameScript(0, s1, 1, s2);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }
        private function toggle():void { clip.gotoAndStop(clip.currentFrame == 1 ? 2 : 1); }
        private function s1():void {
            trace("V1 (goto; Holder with SimpleButton containing Kid in up+over states; A B)");
            toggle(); objs.push(new Holder("V1.H")); objs.push(new Box("V1.A", false)); objs.push(new Box("V1.B", false));
        }
        private function s2():void {
            stop();
            trace("V2 (goto; Box A; Holder; Box B)");
            toggle(); objs.push(new Box("V2.A", false)); objs.push(new Holder("V2.H")); objs.push(new Box("V2.B", false));
        }
        private function onEnterFrame(e:Event):void {
            ticks++;
            var parts:Array = [];
            for each (var b:* in objs) parts.push(b.label + "=" + b.currentFrame);
            Kid.traceRan();
            trace("tick " + ticks + ": " + parts.join(", "));
            if (ticks == 8) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                trace("Done");
            }
        }
    }
}
