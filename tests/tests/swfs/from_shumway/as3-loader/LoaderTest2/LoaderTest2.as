/**
 * Compiled with:
 * node ./utils/compileabc.js --swf LoaderTest2,600,600,60 -p test/swfs/as3-loader/LoaderTest2.as
 */
package {
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.net.URLRequest;
import flash.system.fscommand;

public class LoaderTest2 extends Sprite {

        private var _loader:Loader;

        public function LoaderTest2() {
            trace('loader constructed');
            var basePath : String = stage.loaderInfo.url;
            basePath = basePath.split(/\?#/)[0];
            var pathParts : Array = basePath.split('/');
            pathParts[pathParts.length - 1] = '';
            basePath = pathParts.join('/');
            _loader = new Loader();
            _loader.contentLoaderInfo.addEventListener(Event.OPEN, loader_open);
            _loader.contentLoaderInfo.addEventListener(Event.INIT, loader_init);
            _loader.contentLoaderInfo.addEventListener(Event.COMPLETE, loader_complete);
            // notice no addChild(_loader);
            _loader.load(new URLRequest(basePath + "Loadee2.swf"));
        }

        private function loader_open(event:Event):void {
            trace("loading started");
        }

        private function loader_init(event:Event):void {
            trace("loadee initialized");
        }

        private function loader_complete(event:Event):void {
            trace("loading complete");

	          var loadedSprite = _loader.getChildAt(0);
	          trace("testProperty: " + loadedSprite.testProperty);
	          trace("testSymbol present: " + (loadedSprite.testSymbol != null));

            fscommand('quit');
        }
    }
}
