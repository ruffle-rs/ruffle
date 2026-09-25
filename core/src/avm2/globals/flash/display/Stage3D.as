package flash.display {
    import flash.events.ErrorEvent;
    import flash.events.EventDispatcher;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProfile;
    import flash.utils.setTimeout;

    [API("674")]
    [Ruffle(Abstract)]
    public class Stage3D extends EventDispatcher {
        public native function get context3D():Context3D;

        private native function requestContext3D_internal(context3DRenderMode:String, profiles:Vector.<String>):void;

        public function requestContext3D(context3DRenderMode:String = "auto", profile:String = "baseline"):void {
            this.checkProfile(profile);
            this.requestContext3D_internal(context3DRenderMode, Vector.<String>([profile]));
        }

        [API("692")]
        public function requestContext3DMatchingProfiles(profiles:Vector.<String>):void {
            var profiles = profiles.concat();
            if (profiles.length == 0) {
                throw new ArgumentError("Error #2008: Parameter profiles must be one of the accepted values.", 2008);
            }
            for each (var profile in profiles) {
                this.checkProfile(profile);
            }
            this.requestContext3D_internal("auto", profiles);
        }

        private function checkProfile(profile:String):Boolean {
            var acceptedValues:Array = [
                Context3DProfile.BASELINE,
                Context3DProfile.BASELINE_CONSTRAINED,
                Context3DProfile.BASELINE_EXTENDED,
                Context3DProfile.STANDARD,
                Context3DProfile.STANDARD_CONSTRAINED,
                Context3DProfile.STANDARD_EXTENDED
            ];
            if (acceptedValues.indexOf(profile) == -1) {
                throw new ArgumentError("Error #2008: Parameter profile must be one of the accepted values.", 2008);
            }
        }

        public native function get x():Number;
        public native function set x(value:Number):void;

        public native function get y():Number;
        public native function set y(value:Number):void;

        public native function get visible():Boolean;
        public native function set visible(value:Boolean):void;
    }
}
