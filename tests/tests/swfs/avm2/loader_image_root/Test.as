package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        private var urlLoader:Loader;
        private var bytesLoader:Loader;
        private var dataLoader:URLLoader;

        public function Test() {
            var ordinary:Bitmap = new Bitmap(new BitmapData(1, 1));
            trace("ordinary detached root null:", ordinary.root == null);
            addChild(ordinary);
            trace("ordinary attached root same:", ordinary.root === root);
            removeChild(ordinary);
            trace("ordinary removed root null:", ordinary.root == null);

            urlLoader = new Loader();
            urlLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onUrlComplete);
            urlLoader.load(new URLRequest("test_image.jpg"));
        }

        private function snapshot(label:String, loader:Loader):void {
            trace("=== " + label + " ===");
            trace("loader root null:", loader.root == null);
            trace("loader root same:", loader.root === root);
            trace("content root self:", loader.content.root === loader.content);
            trace("content stage matches loader:", loader.content.stage === loader.stage);
            trace("content loaderInfo same:",
                loader.content.loaderInfo === loader.contentLoaderInfo);
            trace("info content same:",
                loader.contentLoaderInfo.content === loader.content);
        }

        private function probe(label:String, loader:Loader):void {
            snapshot(label + "_DETACHED", loader);
            addChild(loader);
            snapshot(label + "_ATTACHED", loader);
            removeChild(loader);
            snapshot(label + "_REMOVED", loader);
        }

        private function onUrlComplete(event:Event):void {
            probe("URL", urlLoader);

            dataLoader = new URLLoader();
            dataLoader.dataFormat = URLLoaderDataFormat.BINARY;
            dataLoader.addEventListener(Event.COMPLETE, onDataComplete);
            dataLoader.load(new URLRequest("test_image.jpg"));
        }

        private function onDataComplete(event:Event):void {
            var bytes:ByteArray = dataLoader.data as ByteArray;
            bytesLoader = new Loader();
            bytesLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onBytesComplete);
            bytesLoader.loadBytes(bytes);
        }

        private function onBytesComplete(event:Event):void {
            probe("BYTES", bytesLoader);
        }
    }
}
