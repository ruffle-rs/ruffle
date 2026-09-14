package  {
    
    import flash.display.Sprite;
    import flash.display.Loader;
    import flash.display.DisplayObjectContainer;
    import flash.net.URLRequest;
    import flash.events.Event;

    
    public class Test extends Sprite {
        
        var i:Number = 0;
        var loader:Loader = new Loader();
        
        public function Test() {
            super();
            addEventListener("enterFrame", onEnterFrame);
            loader.name = "as3Loader";
            loader.load(new URLRequest("avm1_child.swf"));
            addChild(loader);
        }

        public function onEnterFrame(e:Event) {
            if (i > 3) {
                removeEventListener("enterFrame", onEnterFrame);
                trace("=== avm2 root");

                trace("loader.x: " + loader.x);

                var level0Clip = getChildByName("level0Clip");
                if (!level0Clip) {
                    trace("level0Clip does not exist!");
                } else {
                    trace("getChildByName(\"level0Clip\"): " + level0Clip);
                    trace("level0Clip.x: " + level0Clip.x);
                    trace("getChildIndex(level0Clip): " + getChildIndex(level0Clip));
                }

                // where is as3LoaderClip??? it clearly recognizes it as a child...
                var ctr = (loader as DisplayObjectContainer);
                trace("ctr.getChildAt(0): " + ctr.getChildAt(0));
                trace("ctr.getChildAt(1): " + ctr.getChildAt(1));
                trace("ctr.numChildren: " + ctr.numChildren);
                trace("ctr.getChildByName(\"as3LoaderClip\"): " + ctr.getChildByName("as3LoaderClip"));
            }
            i++;
        }
    }
    
}
