package  {
	
	import flash.display.MovieClip;
	
	
	public class Child extends MovieClip {
		
		public var myDuplicate: MyDuplicate = new ConcreteFromChild();
		
		public function Child() {
			trace("Constructed child: myDuplicate = " + this.myDuplicate);
		}
	}
	
}
