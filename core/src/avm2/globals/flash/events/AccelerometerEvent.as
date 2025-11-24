package flash.events {
    [API("667")]
    public class AccelerometerEvent extends Event {
        public static const UPDATE:String = "update";

        private var _timestamp:Number;
        private var _accelerationX:Number;
        private var _accelerationY:Number;
        private var _accelerationZ:Number;

        public function AccelerometerEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            timestamp:Number = 0,
            accelerationX:Number = 0,
            accelerationY:Number = 0,
            accelerationZ:Number = 0
        ) {
            super(type, bubbles, cancelable);
            this.timestamp = timestamp;
            this.accelerationX = accelerationX;
            this.accelerationY = accelerationY;
            this.accelerationZ = accelerationZ;
        }

        public function get timestamp():Number {
            return this._timestamp;
        }
        public function set timestamp(value:Number):void {
            this._timestamp = value;
        }

        public function get accelerationX():Number {
            return this._accelerationX;
        }
        public function set accelerationX(value:Number):void {
            this._accelerationX = value;
        }

        public function get accelerationY():Number {
            return this._accelerationY;
        }
        public function set accelerationY(value:Number):void {
            this._accelerationY = value;
        }

        public function get accelerationZ():Number {
            return this._accelerationZ;
        }
        public function set accelerationZ(value:Number):void {
            this._accelerationZ = value;
        }

        override public function clone():Event {
            return new AccelerometerEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.timestamp,
                this.accelerationX,
                this.accelerationY,
                this.accelerationZ
            );
        }

        override public function toString():String {
            return this.formatToString(
                "AccelerometerEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "timestamp",
                "accelerationX",
                "accelerationY",
                "accelerationZ"
            );
        }
    }
}
