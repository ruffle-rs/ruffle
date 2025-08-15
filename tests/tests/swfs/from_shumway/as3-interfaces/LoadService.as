/*
 Compiled with:
 node utils/compileabc.js --swf LoadService,100,100,60 -p test/swfs/as3-interfaces/Interface1.as test/swfs/as3-interfaces/LoadService.as
 */

package {
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.net.URLRequest;
import flash.system.fscommand;
import Interface1;

public class LoadService extends Sprite {

  private var _loader:Loader;

  public function LoadService() {
    trace('loading started');
    _loader = new Loader();
    _loader.contentLoaderInfo.addEventListener(Event.COMPLETE, loader_complete);
    addChild(_loader);
    _loader.load(new URLRequest("./Service1.swf"));
  }

  private function loader_complete(event:Event):void {
    trace("loading complete");

    var content: * = _loader.content;
    var service_as: Interface1 = content as Interface1;
    trace('content as Interface1 != null: ' + (service_as != null));

    var service_co: Interface1 = Interface1(content);
    trace(service_co);
    service_co.run();

    fscommand('quit');
  }
}

}