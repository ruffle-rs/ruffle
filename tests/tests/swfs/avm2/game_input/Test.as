package  {
	
	import flash.display.MovieClip;
	import flash.ui.GameInput;
	import flash.ui.GameInputControl;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var gameInput = new GameInput();
			trace("gameInput: " + gameInput);
			trace("GameInput.numDevices: " + GameInput.numDevices);
			try {
				GameInput.getDeviceAt(0);
			} catch (e) {
				trace("Caught error: " + e);
			}
			try {
				new GameInputControl();
			} catch (e) {
				trace("Caught error: " + e);
			}

		}
	}
	
}
