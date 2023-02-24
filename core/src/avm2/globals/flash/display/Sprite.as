package flash.display {

    import flash.geom.Rectangle;
    import flash.media.SoundTransform;

    public class Sprite extends DisplayObjectContainer {

        internal var _graphics:Graphics;

        public function Sprite() {
            this.init();
        }

        private native function init();
        
        public native function get graphics():Graphics;
        public native function get dropTarget():DisplayObject;
        public native function get soundTransform():SoundTransform;
        public native function set soundTransform(sndTransform:SoundTransform):void;
        public native function get buttonMode():Boolean;
        public native function set buttonMode(buttonMode:Boolean):void;
        public native function get useHandCursor():Boolean;
        public native function set useHandCursor(useHandCursor:Boolean):void;

        public native function startDrag(lockCenter:Boolean = false, bounds:Rectangle = null):void;
        public native function stopDrag():void;
    }
}