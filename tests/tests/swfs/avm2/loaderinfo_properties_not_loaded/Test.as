package  {
	
	import flash.display.MovieClip;
	import flash.display.Loader;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
            var loader:Loader = new Loader();
			
			trace("// actionScriptVersion");
			try {
				trace(loader.contentLoaderInfo.actionScriptVersion);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// childAllowsParent");
			try {
				trace(loader.contentLoaderInfo.childAllowsParent);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// frameRate");
			try {
				trace(loader.contentLoaderInfo.frameRate);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// height");
			try {
				trace(loader.contentLoaderInfo.height);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// parentAllowsChild");
			try {
				trace(loader.contentLoaderInfo.parentAllowsChild);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// sameDomain");
			try {
				trace(loader.contentLoaderInfo.sameDomain);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// swfVersion");
			try {
				trace(loader.contentLoaderInfo.swfVersion);
			} catch (error: Error) {
				trace(error);
			}
			trace("");
			
			trace("// width");
			try {
				trace(loader.contentLoaderInfo.width);
			} catch (error: Error) {
				trace(error);
			}
			trace("");

			// constructor code
		}
	}
	
}
