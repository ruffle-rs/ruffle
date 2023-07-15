package flash.display {
    import flash.display.InteractiveObject;
    import flash.geom.Transform;
    import flash.geom.ColorTransform;
    import flash.geom.Matrix;
    import flash.display.DisplayObject;
    import flash.media.SoundTransform;
    
    [Ruffle(InstanceAllocator)]
    public class SimpleButton extends InteractiveObject {
        public function SimpleButton(upState:DisplayObject = null, overState:DisplayObject = null, downState:DisplayObject = null, hitTestState:DisplayObject = null) {
            this.init(upState, overState, downState, hitTestState)
        }

        private native function init(upState:DisplayObject, overState:DisplayObject, downState:DisplayObject, hitTestState:DisplayObject):void;

        public native function get downState():DisplayObject;
        public native function set downState(value:DisplayObject):void;

        public native function get enabled():Boolean;
        public native function set enabled(value:Boolean):void;

        public native function get hitTestState():DisplayObject;
        public native function set hitTestState(value:DisplayObject):void;

        public native function get overState():DisplayObject;
        public native function set overState(value:DisplayObject):void;

        public native function get trackAsMenu():Boolean;
        public native function set trackAsMenu(value:Boolean):void;

        public native function get upState():DisplayObject;
        public native function set upState(value:DisplayObject):void;

        public native function get useHandCursor():Boolean;
        public native function set useHandCursor(value:Boolean):void;

        public native function get soundTransform():SoundTransform;
        public native function set soundTransform(sndTransform:SoundTransform):void;
    }
}