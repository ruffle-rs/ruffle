package {
	import flash.display.MovieClip;
	import flash.display.Stage3D;
	import flash.events.Event;
	
	public class Test {
		public function Test(main:MovieClip) {
			main.stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, function(event) {
				var stage3d : Stage3D = event.target as Stage3D;
				try {
					stage3d.context3D.configureBackBuffer(0, 200, 0);
				} catch (e) {
					trace("Caught error: " + e);
				}
			
				try {
					stage3d.context3D.configureBackBuffer(200, 0, 0);
				} catch (e) {
					trace("Caught error: " + e);
				}
			
				try {
					stage3d.context3D.configureBackBuffer(999999999, 200, 0);
				} catch (e) {
					trace("Caught error: " + e);
				}
			
				try {
					stage3d.context3D.configureBackBuffer(200, 999999999, 0);
				} catch (e) {
					trace("Caught error: " + e);
				}
			
			})
			main.stage.stage3Ds[0].requestContext3D();
		}
	}
}