package {
    import flash.display.Sprite;
    import flash.net.LocalConnection;
    import flash.events.StatusEvent;
    import flash.display.Loader;
    import flash.net.URLRequest;
    import flash.events.Event;
    import flash.utils.ByteArray;
    import flash.events.AsyncErrorEvent;

    public class LCSend extends Sprite {
        private var conn:LocalConnection;
		private var receiver: Loader;
        
        public function LCSend() {
			loadReceiver();
        }
		
		private function loadReceiver() {
			receiver = new Loader();
			addChild(receiver);
			receiver.contentLoaderInfo.addEventListener(Event.INIT, receiver_init);
			receiver.load(new URLRequest('lc-receive.swf'));
		}
		
		private function receiver_init(e: Event) {
            conn = new LocalConnection();
            conn.addEventListener(StatusEvent.STATUS, onStatus);
			conn.addEventListener(AsyncErrorEvent.ASYNC_ERROR, onAsyncError);
			sendMessage();
		}
        
        private function sendMessage():void {
			var payload = "Message sent via LocalConnection";
			trace('Before send');
            conn.send("myConnection_failing", "lcHandler", payload);
            conn.send("myConnection", "lcHandler_failing", payload);
            conn.send("myConnection", "lcHandler", payload);
			trace('After send');
        }
        
        private function onStatus(event:StatusEvent):void {
            switch (event.level) {
                case "status":
                    trace("Second and third sends succeed");
                    break;
                case "error":
                    trace("First send fails, but doesn't throw an asyncError");
                    break;
            }
        }
        
        private function onAsyncError(event:AsyncErrorEvent):void {
			trace('AsyncError caught. Text: ' + event.text + ', error: ' + event.error);
            conn.send("myConnection", "lcHandler", event.toString());
        }
    }
}