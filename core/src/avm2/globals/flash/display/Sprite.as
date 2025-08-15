package flash.display {

    import flash.display.Sprite;
    import flash.geom.Rectangle;
    import flash.media.SoundTransform;

    [Ruffle(InstanceAllocator)]
    public class Sprite extends DisplayObjectContainer {

        [Ruffle(NativeAccessible)]
        private var _graphics:Graphics;

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

        public native function get hitArea():Sprite;
        public native function set hitArea(hitArea:Sprite):void;
    }
}
