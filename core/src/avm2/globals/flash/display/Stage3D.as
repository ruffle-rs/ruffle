package flash.display {
    import flash.events.ErrorEvent;
    import flash.events.EventDispatcher;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProfile;
    import flash.utils.setTimeout;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    [API("674")]
    public class Stage3D extends EventDispatcher {

        public native function get context3D():Context3D;
        private native function requestContext3D_internal(context3DRenderMode:String, profiles:Vector.<String>):void;

        public function requestContext3D(context3DRenderMode:String = "auto", profile:String = "baseline"):void {
            // Several SWFS (the examples from the Context3D documentation, and the Starling framework)
            // rely on the `context3DCreate` being fired asynchronously - they initialize variables
            // after the call to `requestContext3D`, and then use those variables in the event handler.
            // Currently, we create a `Context3D` synchronously, so we need to delay the event dispatch
            var stage3d = this;
            this.checkProfile(profile);
            setTimeout(function() {
                    stage3d.requestContext3D_internal(context3DRenderMode, Vector.<String>([profile]));
                }, 0);
        }

        [API("692")]
        public function requestContext3DMatchingProfiles(profiles:Vector.<String>):void {
            var stage3d = this;
            var profiles = profiles.concat();
            if (profiles.length == 0) {
                throw new ArgumentError("Error #2008: Parameter profiles must be one of the accepted values.", 2008);
            }
            for each (var profile in profiles) {
                this.checkProfile(profile);
            }
            setTimeout(function() {
                    stage3d.requestContext3D_internal("auto", profiles);
                }, 0);
        }

        private function checkProfile(profile:String):Boolean {
            if ([Context3DProfile.BASELINE, Context3DProfile.BASELINE_CONSTRAINED, Context3DProfile.BASELINE_EXTENDED, Context3DProfile.STANDARD, Context3DProfile.STANDARD_CONSTRAINED, Context3DProfile.STANDARD_EXTENDED].indexOf(profile) == -1) {
                throw new ArgumentError("Error #2008: Parameter profile must be one of the accepted values.", 2008);
            }
        }

        // FIXME - actually implement this
        public var x:Number;
        public var y:Number;

        public native function get visible():Boolean;
        public native function set visible(value:Boolean):void;
    }
}