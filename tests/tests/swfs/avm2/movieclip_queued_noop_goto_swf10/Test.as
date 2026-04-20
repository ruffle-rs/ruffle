package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            addFrameScript(0, frame1);

            addEventListener("enterFrame", function(e):void {
                trace("enterFrame");
            });
            addEventListener("frameConstructed", function(e):void {
                trace("frameConstructed");
            });
            addEventListener("exitFrame", function(e):void {
                trace("exitFrame");
            });
        }
        
        public function frame1():void {
            trace("In frame1");
            gotoAndStop(1);
            trace("goto over");
        }
    }
}
