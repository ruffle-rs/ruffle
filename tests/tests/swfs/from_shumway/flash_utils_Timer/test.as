/* 
   Derived from:
   http://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/utils/Timer.html#includeExamplesSummary

   Compiled with:
   java -jar asc.jar -import playerglobal.abc -swf TimerExample,100,100 test/swfs/flash_utils_Timer.as
 */

package {
    import flash.utils.Timer;
    import flash.events.TimerEvent;
    import flash.display.Sprite;

    public class TimerExample extends Sprite {

        public function TimerExample() {
            var myTimer:Timer = new Timer(200, 2);
            myTimer.addEventListener("timer", timerHandler);
            myTimer.start();
        }

        public function timerHandler(event:TimerEvent):void {
            trace("timerHandler: " + event);
            trace("get running: " + event.target.running);
            event.target.stop();
        }
    }
}
