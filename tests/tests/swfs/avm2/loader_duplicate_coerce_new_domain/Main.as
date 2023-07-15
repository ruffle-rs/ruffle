package {
	import flash.events.Event;
	import flash.display.Loader;
	import flash.system.LoaderContext;
	import flash.net.URLRequest;
	import flash.system.ApplicationDomain;
	
	public class Main {
		public function Main() {
			new ConcreteFromMain().fromMain();
			var loader = new Loader();
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				var targetObj: MyDuplicate = loader.content["myDuplicate"];
				trace("targetObj as MyDuplicate: " + (targetObj as MyDuplicate));
				targetObj.fromMain();
			})
			loader.load(new URLRequest("child/child.swf"), new LoaderContext(false, new ApplicationDomain(ApplicationDomain.currentDomain)));
		}
	}
}