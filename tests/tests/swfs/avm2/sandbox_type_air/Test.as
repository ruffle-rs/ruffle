package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.net.URLLoader;
    import flash.net.URLRequest;
    import flash.system.LoaderContext;
    import flash.system.Security;

    public class Test extends MovieClip {
        public function Test() {
            trace("Root sandbox type: " + Security.sandboxType);

            trace("Loading child.swf");
            load(new URLRequest("child.swf"), function():void {
                trace("Loading http://localhost:8000/child.swf");
                load(new URLRequest("http://localhost:8000/child.swf"), function():void {
                    trace("Loading the bytes of child.swf");
                    var urlLoader:URLLoader = new URLLoader();
                    urlLoader.dataFormat = "binary";
                    urlLoader.addEventListener(Event.COMPLETE, function(e:Event):void {
                        var context:LoaderContext = new LoaderContext();
                        context.allowCodeImport = true;
                        var loader:Loader = new Loader();
                        loader.loadBytes(urlLoader.data, context);
                    });
                    urlLoader.load(new URLRequest("child.swf"));
                });
            });
        }

        private function load(request:URLRequest, callback:Function):void {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                callback();
            });
            loader.load(request);
        }
    }
}
