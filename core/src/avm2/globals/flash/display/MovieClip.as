package flash.display {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    public dynamic class MovieClip extends Sprite {
        public function MovieClip() {}

        public native function get currentFrame():int;
        public native function get currentFrameLabel():String;
        public native function get currentLabel():String;
        public native function get currentLabels():Array;
        public native function get currentScene():Scene;
        public native function get enabled():Boolean;
        public native function set enabled(value:Boolean):void;
        public native function get framesLoaded():int;
        [API("674")]
        public native function get isPlaying():Boolean;
        public native function get scenes():Array;
        public native function get totalFrames():int;
        public function get trackAsMenu():Boolean {
            stub_getter("flash.display.MovieClip", "trackAsMenu");
            return false;
        }
        public function set trackAsMenu(value:Boolean):void {
            stub_setter("flash.display.MovieClip", "trackAsMenu");
        }

        public native function addFrameScript(...args):void;
        public native function gotoAndPlay(frame:Object, scene:String = null):void;
        public native function gotoAndStop(frame:Object, scene:String = null):void;
        public native function stop():void;
        public native function play():void;
        public native function prevFrame():void;
        public native function nextFrame():void;
        public native function prevScene():void;
        public native function nextScene():void;
    }
}
