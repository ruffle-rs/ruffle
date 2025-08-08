package  {
	
	import flash.display.MovieClip;
	import flash.display.Loader;
	import flash.net.URLRequest;
	import flash.events.Event;

	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var loader = new Loader();
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				var loadedClass = loader.contentLoaderInfo.applicationDomain.getDefinition("LoadableMain");
				var instance = new loadedClass("From Test Loader!");
				trace("Constructed: " + instance);
				trace("instance.myChild: " + instance.myChild);
			});
			loader.load(new URLRequest("./loadable.swf"));
		}
	}
	
}
