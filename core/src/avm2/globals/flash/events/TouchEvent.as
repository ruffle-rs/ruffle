package flash.events {
    import flash.utils.ByteArray;
    import flash.display.InteractiveObject;
    import __ruffle__.stub_method;

    [API("667")]
    public class TouchEvent extends Event {
        public static const PROXIMITY_BEGIN:String = "proximityBegin";
        public static const PROXIMITY_END:String = "proximityEnd";
        public static const PROXIMITY_MOVE:String = "proximityMove";
        public static const PROXIMITY_OUT:String = "proximityOut";
        public static const PROXIMITY_OVER:String = "proximityOver";
        public static const PROXIMITY_ROLL_OUT:String = "proximityRollOut";
        public static const PROXIMITY_ROLL_OVER:String = "proximityRollOver";
        public static const TOUCH_BEGIN:String = "touchBegin";
        public static const TOUCH_END:String = "touchEnd";
        public static const TOUCH_MOVE:String = "touchMove";
        public static const TOUCH_OUT:String = "touchOut";
        public static const TOUCH_OVER:String = "touchOver";
        public static const TOUCH_ROLL_OUT:String = "touchRollOut";
        public static const TOUCH_ROLL_OVER:String = "touchRollOver";
        public static const TOUCH_TAP:String = "touchTap";

        private var _touchPointID:int;
        private var _isPrimaryTouchPoint:Boolean;
        private var _localX:Number;
        private var _localY:Number;
        private var _sizeX:Number;
        private var _sizeY:Number;
        private var _pressure:Number;
        private var _relatedObject:InteractiveObject;
        private var _ctrlKey:Boolean;
        private var _altKey:Boolean;
        private var _shiftKey:Boolean;
        private var _isRelatedObjectInaccessible:Boolean;
        private var _stageX:Number;
        private var _stageY:Number;

        public function TouchEvent(
            type:String, bubbles:Boolean = true, cancelable:Boolean = false, touchPointID:int = 0,
            isPrimaryTouchPoint:Boolean = false, localX:Number = NaN, localY:Number = NaN,
            sizeX:Number = NaN, sizeY:Number = NaN, pressure:Number = NaN,
            relatedObject: InteractiveObject = null, ctrlKey:Boolean = false,
            altKey:Boolean = false, shiftKey:Boolean = false
        ) {
            super(type, bubbles, cancelable);
            this.touchPointID = touchPointID;
            this.isPrimaryTouchPoint = isPrimaryTouchPoint;
            this.localX = localX;
            this.localY = localY;
            this.sizeX = sizeX;
            this.sizeY = sizeY;
            this.pressure = pressure;
            this.relatedObject = relatedObject;
            this.ctrlKey = ctrlKey;
            this.altKey = altKey;
            this.shiftKey = shiftKey;
        }

        override public function clone():Event {
            return new TouchEvent(this.type, this.bubbles, this.cancelable, this.touchPointID, this.isPrimaryTouchPoint,
                this.localX, this.localY, this.sizeX, this.sizeY, this.pressure, this.relatedObject, this.ctrlKey,
                this.altKey, this.shiftKey);
        }

        [API("675")]
        public function getSamples(buffer: ByteArray, append: Boolean = false):uint {
            stub_method("flash.events.TouchEvent", "getSamples");
            return 0;
        }

        [API("675")]
        public function isToolButtonDown(index: int):Boolean {
            stub_method("flash.events.TouchEvent", "isToolButtonDown");
            return false;
        }

        override public function toString():String {
            return this.formatToString("TouchEvent", "type", "bubbles", "cancelable", "eventPhase", "touchPointID",
                "isPrimaryTouchPoint", "localX", "localY", "stageX", "stageY", "sizeX", "sizeY", "pressure", "relatedObject",
                "ctrlKey", "altKey", "shiftKey");
        }

        public native function updateAfterEvent():void;

        public function get touchPointID():int {
            return this._touchPointID;
        }

        public function set touchPointID(value:int):void {
            this._touchPointID = value;
        }

        public function get isPrimaryTouchPoint():Boolean {
            return this._isPrimaryTouchPoint;
        }

        public function set isPrimaryTouchPoint(value:Boolean):void {
            this._isPrimaryTouchPoint = value;
        }

        public function get localX():Number {
            return this._localX;
        }

        public function set localX(value:Number):void {
            this._localX = value;
        }

        public function get localY():Number {
            return this._localY;
        }

        public function set localY(value:Number):void {
            this._localY = value;
        }

        public function get sizeX():Number {
            return this._sizeX;
        }

        public function set sizeX(value:Number):void {
            this._sizeX = value;
        }

        public function get sizeY():Number {
            return this._sizeY;
        }

        public function set sizeY(value:Number):void {
            this._sizeY = value;
        }

        public function get pressure():Number {
            return this._pressure;
        }

        public function set pressure(value:Number):void {
            this._pressure = value;
        }

        public function get relatedObject():InteractiveObject {
            return this._relatedObject;
        }

        public function set relatedObject(value:InteractiveObject):void {
            this._relatedObject = value;
        }

        public function get ctrlKey():Boolean {
            return this._ctrlKey;
        }

        public function set ctrlKey(value:Boolean):void {
            this._ctrlKey = value;
        }

        public function get altKey():Boolean {
            return this._altKey;
        }

        public function set altKey(value:Boolean):void {
            this._altKey = value;
        }

        public function get shiftKey():Boolean {
            return this._shiftKey;
        }

        public function set shiftKey(value:Boolean):void {
            this._shiftKey = value;
        }

        public function get isRelatedObjectInaccessible():Boolean {
            return this._isRelatedObjectInaccessible;
        }

        public function set isRelatedObjectInaccessible(value:Boolean):void {
            this._isRelatedObjectInaccessible = value;
        }

        public function get stageX():Number {
            return this._stageX;
        }

        public function get stageY():Number {
            return this._stageY;
        }
    }
}
