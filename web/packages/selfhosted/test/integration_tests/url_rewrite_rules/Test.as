package {
    import flash.display.MovieClip;
    import flash.events.KeyboardEvent;
    import flash.display.Loader;
    import flash.net.URLRequest;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="100", height="100")]
    public class Test extends MovieClip {
        public function Test() {
            stage.addEventListener(KeyboardEvent.KEY_DOWN, onKeyDown);
            addChild(createLinkText());

            trace("Loaded test!");
        }

        private function onKeyDown(event: KeyboardEvent):void {
            if (event.charCode == 65) {
                var loader:Loader = new Loader();
                loader.load(new URLRequest("https://example.com/other1.test1"));
                addChild(loader);
            }
            if (event.charCode == 66) {
                var loader:Loader = new Loader();
                loader.load(new URLRequest("other1.test2"));
                addChild(loader);
            }
            if (event.charCode == 67) {
                var loader:Loader = new Loader();
                loader.load(new URLRequest("other2.swf"));
                addChild(loader);
            }
        }

        private function createLinkText():TextField {
            var text:TextField = new TextField();
            text.x = -20;
            text.y = -20;
            text.width = 200;
            text.height = 200;
            text.htmlText = "<font size='100'><a href='http://www.example.com/[test]site.html'>CLICK</a></font>";
            return text;
        }
    }
}
