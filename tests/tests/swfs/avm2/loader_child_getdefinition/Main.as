package  {
	
	import flash.display.MovieClip;
	import flash.system.LoaderContext;
	import flash.system.ApplicationDomain;
	import flash.display.Loader;
	import flash.net.URLRequest;
	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
			new Parent();
			var context = new LoaderContext(false, new ApplicationDomain());
			new Loader().load(new URLRequest("./child/child.swf"), context);
		}
	}
	
}
