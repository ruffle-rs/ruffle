package {
    import flash.display.Sprite;
    import flash.display.Loader;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.events.UncaughtErrorEvent;

    public class Test extends Sprite {
        function Test() {
            trace("new Loader()");
            var loader = new Loader();
            var loaderInfo = loader.contentLoaderInfo;
            trace("loaderInfo.url = " + loaderInfo.url);
            trace("loaderInfo.loaderURL = " + loaderInfo.loaderURL);

            trace("Loader.load()");
            var loader = new Loader();
            loader.load(new URLRequest("/foo.swf"));
            var loaderInfo = loader.contentLoaderInfo;
            trace("loaderInfo.url = " + loaderInfo.url);
            trace("loaderInfo.loaderURL = " + loaderInfo.loaderURL);

            trace("Loader.unload()");
            loader.unload();
            loaderInfo = loader.contentLoaderInfo
            trace("loaderInfo.url = " + loaderInfo.url);
            trace("loaderInfo.loaderURL = " + loaderInfo.loaderURL);

            trace("Loader.loadBytes()");
            var loader = new Loader();
            var bytes = new ByteArray();
            bytes.length = 1;
            loader.loadBytes(bytes);
            var loaderInfo = loader.contentLoaderInfo;
            trace("loaderInfo.url = " + loaderInfo.url);
            trace("loaderInfo.loaderURL = " + loaderInfo.loaderURL);
        }
    }
}

