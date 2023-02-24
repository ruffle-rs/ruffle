package flash.display {
    import flash.accessibility.AccessibilityProperties;
    import flash.filters.BitmapFilter;
    import flash.geom.Rectangle;
    import flash.geom.Transform;
    import flash.ui.ContextMenu;
    import flash.system.LoaderInfo;

    [Ruffle(NativeInstanceInit)]
    public class Stage extends DisplayObjectContainer {

        public function Stage() {
            throw new Error("You cannot construct new instances of the Stage.")
        }

        public native function set accessibilityProperties(value:AccessibilityProperties):void;

        override public native function set alpha(value:Number):void;

        override public native function set blendMode(value:String):void;

        override public native function set cacheAsBitmap(value:Boolean):void;

        override public native function set contextMenu(value:ContextMenu):void;

        override public native function set filters(value:Array):void;

        override public native function set focusRect(value:Object):void;

        public native function set loaderInfo(value:LoaderInfo):void;

        override public native function set mask(value:DisplayObject):void;

        override public native function set mouseEnabled(value:Boolean):void;

        override public native function get name():String;
        override public native function set name(value:String):void;

        override public native function set opaqueBackground(value:Object):void;

        override public native function set rotation(value:Number):void;

        public native function set scale9Grid(value:Rectangle):void;

        override public native function set scaleX(value:Number):void;

        override public native function set scaleY(value:Number):void;

        override public native function set scrollRect(value:Rectangle):void;

        override public native function set tabEnabled(value:Boolean):void;;
 
        override public native function set tabIndex(value:int):void;

        override public native function set transform(value: Transform):void;

        override public native function set visible(value:Boolean):void;

        override public native function set x(value:Number):void;
 
        override public native function set y(value:Number):void;

        // End of overrides

        public native function get align():String;
        public native function set align(value:String):void;

        public native function get browserZoomFactor():Number;

        public native function get color():uint;
        public native function set color(value:uint):void;

        public native function get contentsScaleFactor():Number;

        public native function get displayState():String;
        public native function set displayState(value:String):void;

        public native function get focus():InteractiveObject;
        public native function set focus(value:InteractiveObject):void;

        public native function get frameRate():Number;
        public native function set frameRate(value:Number):void;

        public native function get fullScreenSourceRect():Rectangle;
        public native function set fullScreenSourceRect(value:Rectangle):void;

        public native function get scaleMode():String;
        public native function set scaleMode(value:String):void;

        public native function get showDefaultContextMenu():Boolean;
        public native function set showDefaultContextMenu(value:Boolean):void;

        public native function get stageWidth():Number;
        public native function set stageWidth(value:Number):void;

        public native function get stageHeight():Number;
        public native function set stageHeight(value:Number):void;

        public native function get stageFocusRect():Boolean;
        public native function set stageFocusRect(value:Boolean):void;

        public native function get allowsFullScreen():Boolean;

        public native function get allowsFullScreenInteractive():Boolean;

        public native function get quality():String;
        public native function set quality(value:String):void;

        public native function get stage3Ds():Vector.<Stage3D>;

        public native function invalidate(rect:Rectangle = null):void;
    }
}