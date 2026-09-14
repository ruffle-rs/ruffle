package {
    import flash.display.MovieClip;
    import flash.utils.Timer;

    public class Test extends MovieClip {
        public function Test() {
            testDelay(-1);
            testDelay(NaN);
            testDelay(Infinity);
        }

        function testDelay(delay:Number):void {
            try {
                new Timer(delay);
            } catch (e:*) {
                trace(e.getStackTrace());
            }

            var timer:Timer = new Timer(100);
            try {
                timer.delay = delay;
            } catch (e:*) {
                trace(e.getStackTrace());
            }
        }
    }
}
