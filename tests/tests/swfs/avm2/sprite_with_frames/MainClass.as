package  {
	
	import flash.display.Sprite;
	
	
	public class MainClass extends Sprite {
		
		
		public function MainClass() {
			var child = new MyChild();
			child.x = 100;
			child.y = 50;
			this.addChild(child);
		}
	}
	
}
