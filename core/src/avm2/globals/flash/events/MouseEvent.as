
package flash.events {
    import flash.display.InteractiveObject;

    public class MouseEvent extends Event {
        public static const CLICK:String = "click";
        public static const DOUBLE_CLICK:String = "doubleClick";
        public static const MOUSE_DOWN:String = "mouseDown";
        public static const MOUSE_MOVE:String = "mouseMove";
        public static const MOUSE_OUT:String = "mouseOut";
        public static const MOUSE_OVER:String = "mouseOver";
        public static const MOUSE_UP:String = "mouseUp";
        public static const RELEASE_OUTSIDE:String = "releaseOutside";
        public static const MOUSE_WHEEL:String = "mouseWheel";
        public static const ROLL_OUT:String = "rollOut";
        public static const ROLL_OVER:String = "rollOver";
        [API("678")]
        public static const MIDDLE_CLICK:String = "middleClick";
        [API("678")]
        public static const MIDDLE_MOUSE_DOWN:String = "middleMouseDown";
        [API("678")]
        public static const MIDDLE_MOUSE_UP:String = "middleMouseUp";
        [API("678")]
        public static const RIGHT_CLICK:String = "rightClick";
        [API("678")]
        public static const RIGHT_MOUSE_DOWN:String = "rightMouseDown";
        [API("678")]
        public static const RIGHT_MOUSE_UP:String = "rightMouseUp";
        [API("678")]
        public static const CONTEXT_MENU:String = "contextMenu";

        private var _isRelatedObjectInaccessible:Boolean;
        private var _relatedObject:InteractiveObject;

        [Ruffle(NativeAccessible)]
        private var _localX:Number;

        [Ruffle(NativeAccessible)]
        private var _localY:Number;

        private var _delta:int;
        private var _buttonDown:Boolean;
        private var _altKey:Boolean;
        private var _ctrlKey:Boolean;
        private var _shiftKey:Boolean;
        private var _movementX:Number;
        private var _movementY:Number;

        public function MouseEvent(
            type:String,
            bubbles:Boolean = true,
            cancelable:Boolean = false,
            localX:Number = 0/0,
            localY:Number = 0/0,
            relatedObject:InteractiveObject = null,
            ctrlKey:Boolean = false,
            altKey:Boolean = false,
            shiftKey:Boolean = false,
            buttonDown:Boolean = false,
            delta:int = 0
        ) {
            super(type, bubbles, cancelable);
            this.localX = localX;
            this.localY = localY;
            this.relatedObject = relatedObject;
            this.ctrlKey = ctrlKey;
            this.altKey = altKey;
            this.shiftKey = shiftKey;
            this.buttonDown = buttonDown;
            this.delta = delta;

            this.movementX = 0.0; // unimplemented
            this.movementY = 0.0; // unimplemented
        }

        override public function clone():Event {
            // note: movementX/Y not cloned
            return new MouseEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.localX,
                this.localY,
                this.relatedObject,
                this.ctrlKey,
                this.altKey,
                this.shiftKey,
                this.buttonDown,
                this.delta
            );
        }

        override public function toString():String {
            return this.formatToString(
                "MouseEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "localX",
                "localY",
                "stageX",
                "stageY",
                "relatedObject",
                "ctrlKey",
                "altKey",
                "shiftKey",
                "buttonDown",
                "delta"
            );
        }

        public function get isRelatedObjectInaccessible():Boolean {
            return this._isRelatedObjectInaccessible;
        }

        public function set isRelatedObjectInaccessible(value:Boolean):void {
            this._isRelatedObjectInaccessible = value;
        }

        public function get relatedObject():InteractiveObject {
            return this._relatedObject;
        }

        public function set relatedObject(value:InteractiveObject):void {
            this._relatedObject = value;
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

        public function get delta():int {
            return this._delta;
        }

        public function set delta(value:int):void {
            this._delta = value;
        }

        public function get buttonDown():Boolean {
            return this._buttonDown;
        }

        public function set buttonDown(value:Boolean):void {
            this._buttonDown = value;
        }

        public function get altKey():Boolean {
            return this._altKey;
        }

        public function set altKey(value:Boolean):void {
            this._altKey = value;
        }

        public function get ctrlKey():Boolean {
            return this._ctrlKey;
        }

        public function set ctrlKey(value:Boolean):void {
            this._ctrlKey = value;
        }

        public function get shiftKey():Boolean {
            return this._shiftKey;
        }

        public function set shiftKey(value:Boolean):void {
            this._shiftKey = value;
        }

        [API("678")]
        public function get movementX():Number {
            return this._movementX;
        }

        [API("678")]
        public function set movementX(value:Number):void {
            this._movementX = value;
        }

        [API("678")]
        public function get movementY():Number {
            return this._movementY;
        }

        [API("678")]
        public function set movementY(value:Number):void {
            this._movementY = value;
        }

        public native function updateAfterEvent():void;

        public native function get stageX():Number;
        public native function get stageY():Number;
    }
}
