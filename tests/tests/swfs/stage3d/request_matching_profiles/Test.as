package  {
	
	import flash.display.MovieClip;
	import flash.display.Stage3D;
	import flash.events.Event;
	import flash.display3D.Context3DProfile;
	
	
	public class Test extends MovieClip {
		
		var PROFILES: Array = [Context3DProfile.BASELINE_CONSTRAINED, Context3DProfile.BASELINE, Context3DProfile.BASELINE_EXTENDED, Context3DProfile.STANDARD_CONSTRAINED, Context3DProfile.STANDARD, Context3DProfile.STANDARD_EXTENDED];
		var SUB_VECS: Array = [];
		var stage3D: Stage3D;
		
		public function Test() {
			buildSubArrays(0, Vector.<String>([]));
			this.addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
			this.stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, function(event) {});
		
			try {
				this.stage.stage3Ds[0].requestContext3DMatchingProfiles(Vector.<String>(["baseline", "bad"]));
			} catch (e) {
				trace("Caught: " + e);
			}
		
			try {
				this.stage.stage3Ds[0].requestContext3DMatchingProfiles(Vector.<String>([]));
			} catch (e) {
				trace("Caught: " + e);
			}
		
			try {
				this.stage.stage3Ds[0].requestContext3D("auto", "dummy");
			} catch (e) {
				trace("Caught: " + e);
			}

		}
	
		function buildSubArrays(startPos: int, start: Vector.<String>) {
			for (var i = startPos; i < PROFILES.length; i++) {
				var choice = start.concat();
				choice.push(PROFILES[i]);
				SUB_VECS.push(choice);
				buildSubArrays(i + 1, choice);
			}
		}
	
		private function onEnterFrame(e) {
			if (SUB_VECS.length == 0) {
				this.removeEventListener(Event.ENTER_FRAME, this.onEnterFrame);
				return;
			}
			if (!this.stage3D) {
				var profiles: Vector.<String> = SUB_VECS.pop();
				trace("Requesting profiles: " + profiles);
				this.stage3D = this.stage.stage3Ds[0];
				this.stage3D.requestContext3DMatchingProfiles(profiles);
			} else if (this.stage3D.context3D) {
				trace("Got profile: " + this.stage3D.context3D.profile);
				this.stage3D.context3D.dispose(false);
				trace("After dispose: " + this.stage3D.context3D);
				this.stage3D = null;
			}
		}
	}
	
}
