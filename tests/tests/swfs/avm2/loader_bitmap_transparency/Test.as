package {
  import flash.display.Sprite;
  import flash.display.Loader;
  import flash.events.Event;
  import flash.display.Bitmap;
  import flash.display.LoaderInfo;
  import flash.net.URLRequest;

  public class Test extends Sprite {
    private var files:Array = ["test.jpg","test.png","test_rgba.png"];

    public function Test() {
      super();
      this.loadNext();
    }

    private function loadNext():void {
      var loader:Loader = new Loader();
      loader.contentLoaderInfo.addEventListener(Event.COMPLETE,onLoaded);
      var url:String = files.shift();
      loader.load(new URLRequest(url));
      trace(url);
    }

    private function onLoaded(event:Event):void {
      var loaderInfo:LoaderInfo = event.target as LoaderInfo;
      trace(loaderInfo.loader.content,loaderInfo.contentType);
      var bitmap:Bitmap = Bitmap(loaderInfo.loader.content);
      trace("transparent:");
      trace(bitmap.bitmapData.transparent);
      trace();

      if(files.length != 0) {
        this.loadNext();
      }
    }

  }
}
