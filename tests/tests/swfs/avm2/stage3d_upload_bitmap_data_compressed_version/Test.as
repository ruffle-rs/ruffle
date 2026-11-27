package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.events.Event;
    import flash.net.URLRequest;

    // Flash Player only allows uploading a BitmapData to a compressed texture
    // when the root SWF is version 17 or later. This SWF is version 16, and
    // child.swf is version 17, which doesn't make a difference.
    public class Test extends MovieClip {
        public function Test() {
            var stage3D:Stage3D = stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, function(event:Event):void {
                stage3D.context3D.enableErrorChecking = true;
                stage3D.context3D.configureBackBuffer(64, 64, 0, false);
                Uploads.run("root SWF 16", stage3D.context3D);

                var loader:Loader = new Loader();
                addChild(loader);
                loader.load(new URLRequest("child.swf"));
            });
            stage3D.requestContext3D("auto", "standard");
        }
    }
}
