package {

import flash.display.Sprite;
import flash.display.Loader;
import flash.display.AVM1Movie;
import flash.net.URLRequest;

public class Test extends Sprite {
	var loader = new Loader();

	public function Test() {
    	loader.contentLoaderInfo.addEventListener("complete", onLoaded);
    	loader.load(new URLRequest("child.swf"));
	}

	private function onLoaded(e:*) {
		var mov = (loader.content as AVM1Movie);
		addChild(mov);

		try {
			mov.call();
		} catch (e) { trace(e.getStackTrace()); }

		try {
			mov.addCallback();
		} catch (e) { trace(e.getStackTrace()); }

		try {
			mov.addCallback("hello");
		} catch (e) { trace(e.getStackTrace()); }

		try {
			mov.addCallback("hello", function() {});
		} catch (e) { trace(e.getStackTrace()); }

		try {
			mov.call("hello");
		} catch (e) { trace(e.getStackTrace()); }
	}
}

}
