/**
 * Compiled with:
 * java -jar utils/asc.jar -import build/playerglobal/playerglobal-single.abc -swf Loadee,600,600 test/swfs/as3-loader/Loadee.as
 */

package {
import flash.display.Sprite;
    import flash.events.Event;

    public class Loadee extends Sprite {
        public function Loadee() {
            trace('from loadee: loaded');
            addEventListener(Event.ENTER_FRAME, self_enterFrame);
        }

        private function self_enterFrame(event:Event):void {
            const color : uint = (Math.random() * 0xff) << 16 | (Math.random() * 0xff) << 8 | (Math.random() * 0xff);
            graphics.clear();
            graphics.beginFill(color);
            graphics.drawRect(0, 0, 100, 100);
        }
}
}
