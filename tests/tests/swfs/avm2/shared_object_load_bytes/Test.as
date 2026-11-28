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

    // SharedObject.getLocal called by AVM2 and AVM1 movies loaded with
    // Loader.loadBytes, by the root, by a movie in another directory and by
    // a movie that was itself loaded with Loader.loadBytes.
    public class Test extends Sprite {
        private var middle:Object;
        private var rootChild:Object;
        private var middleChild:Object;
        private var childBytes:ByteArray;
        private var avm1ChildBytes:ByteArray;
        private var objects:Array = [];
        private var labels:Array = [];

        public function Test() {
            check(getLocal, "root");
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                middle = loader.content;
            });
            loader.load(new URLRequest("sub/loader.swf"));
            fetch("child.swf", function(bytes:ByteArray):void {
                childBytes = bytes;
            });
            fetch("child_avm1.swf", function(bytes:ByteArray):void {
                avm1ChildBytes = bytes;
            });
            setTimeout(step1, 300);
        }

        private function getLocal(name:String):SharedObject {
            return SharedObject.getLocal(name);
        }

        private function fetch(url:String, callback:Function):void {
            var loader:URLLoader = new URLLoader();
            loader.dataFormat = URLLoaderDataFormat.BINARY;
            loader.addEventListener(Event.COMPLETE, function(e:Event):void {
                callback(loader.data);
            });
            loader.load(new URLRequest(url));
        }

        private function loadBytes(bytes:ByteArray, callback:Function):void {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                callback(loader.content);
            });
            loader.loadBytes(bytes);
        }

        private function step1():void {
            check(middle.getLocal, "sub/loader.swf");
            loadBytes(childBytes, function(child:Object):void {
                rootChild = child;
                urls(child, "child loaded by root");
                check(child.getLocal, "child loaded by root");
                middle.loadBytes(childBytes, function(child:Object):void {
                    middleChild = child;
                    urls(child, "child loaded by sub/loader.swf");
                    check(child.getLocal, "child loaded by sub/loader.swf");
                    rootChild.loadBytes(childBytes, function(child:Object):void {
                        urls(child, "child loaded by child loaded by root");
                        check(child.getLocal, "child loaded by child loaded by root");
                        middleChild.loadBytes(childBytes, function(child:Object):void {
                            urls(child, "child loaded by child loaded by sub/loader.swf");
                            check(child.getLocal, "child loaded by child loaded by sub/loader.swf");
                            setTimeout(step2, 300);
                        });
                    });
                });
            });
        }

        private function step2():void {
            trace("// AVM1 child loaded by root");
            loadBytes(avm1ChildBytes, function(child:Object):void {});
            setTimeout(step3, 300);
        }

        private function step3():void {
            trace("// AVM1 child loaded by sub/loader.swf");
            middle.loadBytes(avm1ChildBytes, function(child:Object):void {});
            setTimeout(step4, 300);
        }

        private function step4():void {
            trace("// AVM1 child loaded by child loaded by root");
            rootChild.loadBytes(avm1ChildBytes, function(child:Object):void {});
            setTimeout(step5, 300);
        }

        private function step5():void {
            trace("// AVM1 child loaded by child loaded by sub/loader.swf");
            middleChild.loadBytes(avm1ChildBytes, function(child:Object):void {});
            setTimeout(step6, 300);
        }

        private function step6():void {
            trace("// Objects written by the AVM1 children");
            trace("root: " + getLocal("avm1").data.count);
            trace("sub/loader.swf: " + middle.getLocal("avm1").data.count);
        }

        // Prints a URL relative to the directory of the root movie.
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
