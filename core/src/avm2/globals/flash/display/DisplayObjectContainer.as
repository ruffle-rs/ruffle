package flash.display {
    import flash.accessibility.AccessibilityProperties;
    import flash.geom.ColorTransform;
    import flash.geom.Matrix;
    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.media.SoundTransform;
    import flash.display.DisplayObject;
    import flash.display.InteractiveObject;
    import flash.text.TextSnapshot;

    [Ruffle(SuperInitializer)]
    public class DisplayObjectContainer extends InteractiveObject {

        public function DisplayObjectContainer() {
            throw new Error("You cannot construct DisplayObjectContainer directly.");
        }

        public native function get numChildren():int;
        public native function get mouseChildren():Boolean;
        public native function set mouseChildren(value:Boolean):void;
        public native function get tabChildren():Boolean;
        public native function set tabChildren(value:Boolean):void;

        public native function addChild(child:DisplayObject):DisplayObject;
        public native function addChildAt(child:DisplayObject, index:int):DisplayObject;
        public native function contains(child:DisplayObject):Boolean;
        public native function getChildAt(index:int):DisplayObject;
        public native function getChildByName(name:String):DisplayObject;
        public native function getChildIndex(child:DisplayObject):int;
        public native function removeChild(child:DisplayObject):DisplayObject;
        public native function removeChildAt(index:int):DisplayObject;

        [API("674")] // AIR 3.0, FP 11, SWF 13
        public native function removeChildren(beginIndex:int = 0, endIndex:int = 0x7fffffff):void;

        public native function setChildIndex(child:DisplayObject, index:int):void;
        public native function swapChildren(child1:DisplayObject, child2:DisplayObject):void;
        public native function swapChildrenAt(index1:int, index2:int):void;

        [API("690")] // AIR 3.8, FP 11.8, SWF 21
        public native function stopAllMovieClips():void;

        public native function getObjectsUnderPoint(point:Point):Array;
        public native function areInaccessibleObjectsUnderPoint(point:Point):Boolean;

        public function get textSnapshot():TextSnapshot {
            stub_getter("flash.display.DisplayObjectContainer", "textSnapshot")
            return new TextSnapshot();
        }
    }
}
