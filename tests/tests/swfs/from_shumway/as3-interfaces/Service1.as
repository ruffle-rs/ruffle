/*
 Compiled with:
 node utils/compileabc.js --swf Service1,100,100,60 -p test/swfs/as3-interfaces/Interface1.as test/swfs/as3-interfaces/Service1.as
 */

package {
import flash.display.Sprite;
import flash.events.Event;
import Interface1;

public class Service1 extends Sprite implements Interface1 {
  public function Service1() {
    trace('service initialized');
  }

  public function run(): void {
    trace('service: run()');
    graphics.clear();
    graphics.beginFill( 0x00FF00);
    graphics.drawRect(0, 0, 100, 100);
  }
}
}