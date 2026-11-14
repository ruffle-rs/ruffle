package flash.events {
    [API("668")]
    public class GeolocationEvent extends Event {
        public static const UPDATE:String = "update";

        private var _latitude:Number;
        private var _longitude:Number;
        private var _altitude:Number;
        private var _horizontalAccuracy:Number;
        private var _verticalAccuracy:Number;
        private var _speed:Number;
        private var _heading:Number;
        private var _timestamp:Number;

        public function GeolocationEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            latitude:Number = 0,
            longitude:Number = 0,
            altitude:Number = 0,
            hAccuracy:Number = 0,
            vAccuracy:Number = 0,
            speed:Number = 0,
            heading:Number = 0,
            timestamp:Number = 0
        ) {
            super(type, bubbles, cancelable);
            this._latitude = latitude;
            this._longitude = longitude;
            this._altitude = altitude;
            this._horizontalAccuracy = hAccuracy;
            this._verticalAccuracy = vAccuracy;
            this._speed = speed;
            this._heading = heading;
            this._timestamp = timestamp;
        }

        public function get latitude():Number {
            return this._latitude;
        }
        public function set latitude(value:Number):void {
            this._latitude = value;
        }

        public function get longitude():Number {
            return this._longitude;
        }
        public function set longitude(value:Number):void {
            this._longitude = value;
        }

        public function get altitude():Number {
            return this._altitude;
        }
        public function set altitude(value:Number):void {
            this._altitude = value;
        }

        public function get horizontalAccuracy():Number {
            return this._horizontalAccuracy;
        }
        public function set horizontalAccuracy(value:Number):void {
            this._horizontalAccuracy = value;
        }

        public function get verticalAccuracy():Number {
            return this._verticalAccuracy;
        }
        public function set verticalAccuracy(value:Number):void {
            this._verticalAccuracy = value;
        }

        public function get speed():Number {
            return this._speed;
        }
        public function set speed(value:Number):void {
            this._speed = value;
        }

        public function get heading():Number {
            return this._heading;
        }
        public function set heading(value:Number):void {
            this._heading = value;
        }

        public function get timestamp():Number {
            return this._timestamp;
        }
        public function set timestamp(value:Number):void {
            this._timestamp = value;
        }

        override public function clone():Event {
            return new GeolocationEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this._latitude,
                this._longitude,
                this._altitude,
                this._horizontalAccuracy,
                this._verticalAccuracy,
                this._speed,
                this._heading,
                this._timestamp
            );
        }

        override public function toString():String {
            return this.formatToString(
                "GeolocationEvent",
                "type",
                "bubbles",
                "cancelable",
                "latitude",
                "longitude",
                "altitude",
                "horizontalAccuracy",
                "verticalAccuracy",
                "speed",
                "heading",
                "timestamp"
            );
        }
    }
}
