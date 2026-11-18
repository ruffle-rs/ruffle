package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.SharedObject;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.setTimeout;

    // SharedObject.getLocal called by movies loaded with Loader.loadBytes by
    // movies whose URLs contain "/[[DYNAMIC]]/".
    public class Test extends Sprite {
        private var paths:Array = ["[[DYNAMIC]]/1/loader.swf", "[[DYNAMIC]]/2"];
        private var middles:Array = [];
        private var childBytes:ByteArray;
        private var objects:Array = [];
        private var labels:Array = [];

        public function Test() {
            check(getLocal, "root");
            for each (var path:String in paths) {
                load(path);
            }
            var loader:URLLoader = new URLLoader();
            loader.dataFormat = URLLoaderDataFormat.BINARY;
            loader.addEventListener(Event.COMPLETE, function(e:Event):void {
                childBytes = loader.data;
            });
            loader.load(new URLRequest("child.swf"));
            setTimeout(step1, 300);
        }

        private function getLocal(name:String):SharedObject {
            return SharedObject.getLocal(name);
        }

        private function load(path:String):void {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                middles[paths.indexOf(path)] = loader.content;
            });
            loader.load(new URLRequest(path));
        }

        private function step1():void {
            for (var i:int = 0; i < paths.length; i++) {
                urls(middles[i], paths[i]);
                check(middles[i].getLocal, paths[i]);
            }
            loadChild(0);
        }

        private function loadChild(i:int):void {
            if (i == paths.length) {
                return;
            }
            middles[i].loadBytes(childBytes, function(child:Object):void {
                var label:String = "child loaded by " + paths[i];
                urls(child, label);
                check(child.getLocal, label);
                loadChild(i + 1);
            });
        }

        // Prints URLs relative to the directory of the root movie.
        private function urls(child:Object, label:String):void {
            var root:String = loaderInfo.url;
            var base:String = root.substring(0, root.lastIndexOf("/") + 1);
            var info:Object = child.loaderInfo;
            trace(label + ": url " + info.url.split(base).join("") + ", loaderURL " + info.loaderURL.split(base).join(""));
        }

        private function check(getter:Function, label:String):void {
            var so:SharedObject = getter("so");
            var index:int = objects.indexOf(so);
            if (index >= 0) {
                trace(label + ": same as " + labels[index]);
            } else {
                objects.push(so);
                labels.push(label);
                trace(label + ": new");
            }
            so.data.writer = label;
            trace(label + ": " + so.flush());
        }
    }
}
