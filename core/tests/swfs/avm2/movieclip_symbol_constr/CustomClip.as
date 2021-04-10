package  {
	import flash.display.MovieClip;
	
	public class CustomClip extends MovieClip {

		public function CustomClip() {
			trace("//(in CustomClip constr...)");
			trace("//this.name");
			trace(this.name);
			trace("//this.parent");
			trace(this.parent);
		}

	}
	
}
