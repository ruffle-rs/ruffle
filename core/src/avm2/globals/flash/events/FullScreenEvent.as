package flash.events {
    public class FullScreenEvent extends ActivityEvent {

        public static const FULL_SCREEN:String = "fullScreen";
        public static const FULL_SCREEN_INTERACTIVE_ACCEPTED:String = "fullScreenInteractiveAccepted";


        private var _fullScreen:Boolean;
        private var _interactive:Boolean;

        public function FullScreenEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, fullScreen:Boolean = false, interactive:Boolean = false)
        {
            super(type,bubbles,cancelable);
            this._fullScreen = fullScreen;
            this._interactive = interactive;
        }

        override public function clone() : Event
        {
            return new FullScreenEvent(this.type,this.bubbles,this.cancelable,this.fullScreen,this.interactive);
        }

        override public function toString() : String
        {
            return this.formatToString("FullScreenEvent","type","bubbles","cancelable","eventPhase","fullScreen","interactive");
        }

        public function get fullScreen() : Boolean
        {
            return this._fullScreen;
        }

        [API("680")]
        public function get interactive() : Boolean
        {
            return this._interactive;
        }
    }
}
