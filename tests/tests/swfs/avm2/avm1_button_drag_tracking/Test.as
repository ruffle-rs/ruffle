package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.display.Sprite;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.events.KeyboardEvent;
    import flash.events.MouseEvent;
    import flash.net.URLRequest;
    import flash.text.TextField;

    [SWF(width="550", height="400", frameRate="24", backgroundColor="#DCE7F5")]
    public class Test extends MovieClip {
        private var output:TextField = new TextField();
        private var enters:int = 0;
        private var presses:int = 0;
        private var releases:int = 0;

        public function Test() {
            stage.align = StageAlign.TOP_LEFT;
            stage.scaleMode = StageScaleMode.NO_SCALE;

            var loader:Loader = new Loader();
            loader.load(new URLRequest("avm1.swf"));
            addChild(loader);

            var host:Sprite = new Sprite();
            host.graphics.beginFill(0x6699CC);
            host.graphics.drawRect(380, 20, 100, 200);
            host.graphics.endFill();
            host.buttonMode = true;
            for each (var type:String in [MouseEvent.ROLL_OVER,
                    MouseEvent.MOUSE_DOWN, MouseEvent.MOUSE_UP]) {
                host.addEventListener(type, log);
            }
            addChild(host);

            output.x = 500;
            output.y = 20;
            output.width = 200;
            output.height = 500;
            output.mouseEnabled = false;
            addChild(output);
            stage.addEventListener(KeyboardEvent.KEY_UP, function(event:KeyboardEvent):void {
                if (event.keyCode == 32) {
                    output.text = "host " + enters + "/" + presses + "/" + releases;
                    trace(output.text);
                    enters = presses = releases = 0;
                }
            });
        }

        private function log(event:MouseEvent):void {
            if (event.type == MouseEvent.ROLL_OVER) enters++;
            if (event.type == MouseEvent.MOUSE_DOWN) presses++;
            if (event.type == MouseEvent.MOUSE_UP) releases++;
        }
    }
}
