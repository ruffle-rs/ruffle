 package {
   import flash.display.Loader;
   import flash.display.Sprite;
   import flash.net.URLRequest;
   import flash.system.ApplicationDomain;
   import flash.system.LoaderContext;
   
   public class Test extends Sprite {
      public function Test() {
         var appDomain:ApplicationDomain = new ApplicationDomain();
         var loader:Loader = new Loader();
         trace(loader.contentLoaderInfo.applicationDomain);
         loader.load(new URLRequest("loadable.swf"), new LoaderContext(null, appDomain));
         trace(loader.contentLoaderInfo.applicationDomain);
         loader.contentLoaderInfo.addEventListener("complete", function(e:*) {
            trace(loader.contentLoaderInfo.applicationDomain);
            loader.unload();
            trace(loader.contentLoaderInfo.applicationDomain);
            trace(loader.contentLoaderInfo.bytesLoaded);
            trace(loader.contentLoaderInfo.bytesTotal);
         });
         addChild(loader);
      }
   }
}

