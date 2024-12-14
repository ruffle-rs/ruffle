package  {
	
	import flash.display.SimpleButton;
	
	
	public class MyButton extends SimpleButton {
		
		
		public function MyButton() {
			trace("Calling MyButton " + this.name + " super(): this.parent[\"myButton\"] = " + this.parent["myButton"]);
			super();
			trace("Called MyButton " + this.name + " super(): this.parent[\"myButton\"] = " + this.parent["myButton"]);
		}
	}
	
}
