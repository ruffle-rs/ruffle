package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.net.SharedObject;
    import flash.net.URLRequest;
    import flash.utils.setTimeout;

    // SharedObject.getLocal called by a function of a child movie, after the
    // loader that held the child loaded another movie, or failed to.
    public class Test extends Sprite {
        private var urls:Array = [
            "foo:/nonexistent/a.swf",
            "foo:nonexistent/a.swf",
            "foo:///nonexistent/a.swf",
            "foo://host/nonexistent/a.swf",
            "nonexistent/a.swf",
            "other/child.swf"
        ];
        private var localPaths:Array = [null, "/"];
        private var loaders:Array = [];
        private var saved:Array = [];
        private var events:Array = [];
        private var objects:Array = [];
        private var labels:Array = [];

        public function Test() {
            trace("// Before the loaders load another movie");
            check(getLocal, "root");
            for (var i:int = 0; i < urls.length; i++) {
                loaders[i] = load("sub/child.swf");
            }
            setTimeout(step1, 300);
        }

        private function getLocal(name:String, localPath:String):SharedObject {
            return SharedObject.getLocal(name, localPath);
        }

        private function load(url:String):Loader {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                events.push(url + ": complete");
            });
            loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, function(e:Event):void {
                events.push(url + ": ioError");
            });
            try {
                loader.load(new URLRequest(url));
            } catch (e:Error) {
                events.push(url + ": " + Object(e).constructor);
            }
            return loader;
        }

        private function step1():void {
            check(loaders[0].content.getLocal, "child");
            events = [];
            for (var i:int = 0; i < urls.length; i++) {
                saved[i] = loaders[i].content.getLocal;
                loaders[i].unload();
                loaders[i] = load(urls[i]);
            }
            setTimeout(step2, 300);
        }

        private function step2():void {
            trace("// After the loaders loaded another movie, or failed to");
            trace(events.sort().join("\n"));
            for (var i:int = 0; i < urls.length; i++) {
                check(saved[i], "child, after loader" + i + " loaded");
            }
        }

        private function check(getter:Function, label:String):void {
            for each (var localPath:String in localPaths) {
                var itemLabel:String = label + ", " + localPath;
                var so:SharedObject;
                try {
                    so = getter("so", localPath);
                } catch (e:Error) {
                    trace(itemLabel + ": " + Object(e).constructor);
                    continue;
                }
                var index:int = objects.indexOf(so);
                if (index >= 0) {
                    trace(itemLabel + ": same as " + labels[index]);
                    continue;
                }
                objects.push(so);
                labels.push(itemLabel);
                so.data.label = itemLabel;
                trace(itemLabel + ": new, " + so.flush());
            }
        }
    }
}
