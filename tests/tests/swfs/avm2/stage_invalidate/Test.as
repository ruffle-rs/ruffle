package {
    import flash.display.Sprite;
    import flash.events.Event;
    
    public class Test extends Sprite {
        private var test_stage: int = 0;
        
        public function Test() {
            addEventListener(Event.ENTER_FRAME, enterFrameListener);
            addEventListener(Event.FRAME_CONSTRUCTED, frameConstructedListener);
            addEventListener(Event.EXIT_FRAME, exitFrameListener);
            addEventListener(Event.RENDER, renderListener);
            test_stage = 0;
        }
        
        public function enterFrameListener(e: Event) : void {
            trace("ENTER_FRAME ( test_stage = ", test_stage, " )");
            if ((test_stage == 2) || (test_stage == 14) || (test_stage == 17)) {
                trace("Invalidate called inside enterFrameListener");
                stage.invalidate();
            }
            test_stage = (test_stage + 1) % 21;
        }

        public function frameConstructedListener(e: Event) : void {
            trace("FRAME_CONSTRUCTED ( test_stage = ", test_stage, " )");
            if ((test_stage == 6) || (test_stage == 12) || (test_stage == 15)) {
                trace("Invalidate called inside frameConstructedListener");
                stage.invalidate();
            }
            test_stage = (test_stage + 1) % 21;
        }
        
        public function exitFrameListener(e: Event) : void {
            trace("EXIT_FRAME ( test_stage = ", test_stage, " )");
            if ((test_stage == 10) || (test_stage == 13) || (test_stage == 19)) {
                trace("Invalidate called inside exitFrameListener");
                stage.invalidate();
            }
            test_stage = (test_stage + 1) % 21;
        }
        
        public function renderListener(e: Event) : void {
            trace("RENDER ( test_stage = ", test_stage, " )");
        }
    }
}
