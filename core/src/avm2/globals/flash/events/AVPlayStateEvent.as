package flash.events {
    import flash.media.AVPlayState;

    [API("688")]
    public class AVPlayStateEvent extends Event {
        public static const AV_PLAY_STATE:String = "avPlayState";

        private var _playState:AVPlayState;

        public function AVPlayStateEvent(type:String = "avPlayState", bubbles:Boolean = false, cancelable:Boolean = false, playState:int = 0) {
            super(type, bubbles, cancelable);
            this._playState = new AVPlayState(playState);
        }

        public function get playState():AVPlayState {
            return this._playState;
        }
    }
}
