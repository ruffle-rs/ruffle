package {
	import flash.display.MovieClip;
	import flash.display.Stage3D;
	import flash.events.Event;
	
	public class Test {
		public function Test(main:MovieClip) {
			main.stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, function(event) {
				var stage3d : Stage3D = event.target as Stage3D;

				stage3d.context3D.setProgram(null);
				
				try {
					stage3d.context3D.configureBackBuffer(0, 0, 0, false);
				} catch (e) {
					trace("Caught error with all 0 or false: " + e);
				}

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
			
				stage3d.context3D.setVertexBufferAt(0, null, 0, "Dummy format");
				var buffer = stage3d.context3D.createVertexBuffer(1, 1);
				try {
					stage3d.context3D.setVertexBufferAt(0, buffer, 0, "Bad format");
				} catch (e) {
					trace("Caught error: " + e);
				}
			
				trace("Done")
			
			})
			main.stage.stage3Ds[0].requestContext3D();
		}
	}
}