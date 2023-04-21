package  {
	
	import flash.display.MovieClip;
	import flash.display.MorphShape;
	import flash.utils.getQualifiedClassName;
	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
			try {
				new MorphShape();
			} catch (e) {
				trace("Caught error: " + e + " code: " + e.errorID);
			}
		
			var child = this.getChildAt(0);
			trace("Child: " + child + " " + getQualifiedClassName(child));
		}
	}
	
}
