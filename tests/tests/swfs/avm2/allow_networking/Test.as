package {
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.net.*;
    import flash.media.Sound;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        public function Test() {
            // allowNetworking="internal"

            check("navigateToURL", function():void {
                navigateToURL(new URLRequest("https://example.com/"), "_blank");
            });

            check("fscommand", function():void {
                fscommand("test", "argument");
            });

            // allowNetworking="none"

            check("sendToURL", function() {
                sendToURL(new URLRequest("https://example.com/"));
            });

            check("new URLLoader", function() {
                new URLLoader(new URLRequest("http://example1.com"));
            });

            check("URLLoader.load", function() {
                var loader = new URLLoader();
                loader.load(new URLRequest("http://example2.com"));
            });

            check("URLStream.load", function() {
                var stream = new URLStream();
                stream.load(new URLRequest("http://example3.com"));
            });

            check("Loader.load", function() {
                var loader = new Loader();
                loader.load(new URLRequest("http://example4.com"));
            });

            check("new Sound", function() {
                new Sound(new URLRequest("http://example5.com"));
            });

            check("Sound.load", function() {
                var sound = new Sound();
                sound.load(new URLRequest("http://example6.com"));
            });

            check("Sound.load(null)", function() {
                var sound = new Sound();
                sound.load(null);
            });

            check("new Socket", function() {
                new Socket("localhost", 8000);
            });

            check("Socket.connect", function() {
                var socket = new Socket();
                socket.connect("example.com", 8000);
            });

            check("new XMLSocket", function() {
                new XMLSocket("8.8.8.8", 8000);
            });

            check("XMLSocket.connect", function() {
                var socket = new XMLSocket();
                socket.connect("localhost", 8000);
            });

            check("SharedObject.getLocal", function() {
                SharedObject.getLocal("Test");
            });
        }

        private function check(name:String, f:Function):void {
            trace("TEST: " + name);
            try {
                f();
                trace("no error");
            } catch (error:*) {
                trace(error.constructor);
                trace(error.getStackTrace());
            }
            trace("END");
        }
    }
}
