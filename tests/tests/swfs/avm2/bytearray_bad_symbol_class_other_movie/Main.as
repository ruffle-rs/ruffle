package {
import flash.display.Loader;
import flash.display.Sprite;
import flash.events.Event;
import flash.net.URLLoader;
import flash.net.URLRequest;
import flash.system.ApplicationDomain;

public class Main extends Sprite {
	public function Main() {
		trace("///new URLLoader()");
		var urlLoader:URLLoader = new URLLoader();
		urlLoader.dataFormat = "binary";
		urlLoader.addEventListener(Event.COMPLETE, function(e:Event):void {
			trace("///new Loader()");
			var loader:Loader = new Loader();

			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
				var domain:ApplicationDomain = loader.contentLoaderInfo.applicationDomain;
				var MyBytesClass:Class = domain.getDefinition("MyBytes") as Class;
				trace("///new MyBytesClass()");
				var b:* = new MyBytesClass();
				trace("created: " + b);
			});

			loader.loadBytes(urlLoader.data);
		});
		urlLoader.load(new URLRequest("loadable.swf"));
	}
}
}
