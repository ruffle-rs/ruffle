package  {
	
	import flash.display.MovieClip;
	import flash.display.Stage3D;
	import flash.events.Event;
	import flash.display3D.Context3DProfile;
	
	
	public class Test extends MovieClip {
		
		var PROFILES: Array = [Context3DProfile.BASELINE_CONSTRAINED, Context3DProfile.BASELINE, Context3DProfile.BASELINE_EXTENDED, Context3DProfile.STANDARD_CONSTRAINED, Context3DProfile.STANDARD, Context3DProfile.STANDARD_EXTENDED];
		var stage3D: Stage3D;
		
		public function Test() {
			this.addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
			this.stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, function(event) {});
			/*try {
				this.stage.stage3Ds[0].requestContext3D("auto", "bad");
			} catch (e) {
				trace("Caught e: " + e);
			}*/

		}
	
		private function onEnterFrame(e) {
			if (PROFILES.length == 0) {
				this.removeEventListener(Event.ENTER_FRAME, this.onEnterFrame);
				return;
			}
			if (!this.stage3D) {
				var profile = PROFILES.pop();
				trace("Requesting profile: " + profile);
				this.stage3D = this.stage.stage3Ds[0];
				this.stage3D.requestContext3D("auto", profile);
			} else if (this.stage3D.context3D) {
				trace("Got profile: " + this.stage3D.context3D.profile);
				this.stage3D.context3D.dispose(false);
				trace("After dispose: " + this.stage3D.context3D);
				this.stage3D = null;
			}
		}
	}
	
}
