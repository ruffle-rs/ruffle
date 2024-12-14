package {
	
	import flash.display.MovieClip;
	import flash.utils.getDefinitionByName;
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			trace("IFilePromise: " + getDefinitionByName("flash.desktop.IFilePromise"));
		}
	}
	
}