// Code in LocalConnectionReceiverExample.as
package {
    import flash.display.Sprite;
    import flash.net.LocalConnection;
    import flash.text.TextField;
    import flash.text.StyleSheet;
    import flash.events.AsyncErrorEvent;

    public class LCReceive extends Sprite {
        private var conn:LocalConnection;
        private var output:TextField;
        
        public function LCReceive()     {
            buildUI();
            
            conn = new LocalConnection();
			conn.addEventListener(AsyncErrorEvent.ASYNC_ERROR, onAsyncError);
            conn.client = this;
            try {
				trace('Before connect');
                conn.connect("myConnection");
				trace('After connect');
				trace('Connecting a second time fails:');
				conn.connect('myConnection1');
            } catch (error:ArgumentError) {
				trace(error);
            }
        }
        
        public function lcHandler(msg:String):void {
            output.appendText(msg + "\n");
			trace('Received via LocalConnection: ' + msg);
			throw new Error('fooo');
        }
        
        private function buildUI():void {
            output = new TextField();
			output.width = 300;
            output.background = true;
            output.border = true;
            output.wordWrap = true;
            addChild(output);
        }
        
        private function onAsyncError(event:AsyncErrorEvent):void {
			trace('AsyncError caught. Text: ' + event.text + ', error: ' + event.error);
        }
    }
}