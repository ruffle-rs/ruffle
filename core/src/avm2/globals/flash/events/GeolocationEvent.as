package flash.events
{

    public class GeolocationEvent extends Event
    {
        // [static] Defines the value of the type property of a GeolocationEvent event object.
        public static const UPDATE:String = "update";

        // The latitude in degrees.
        public var latitude: Number;

        // The longitude in degrees.
        public var longitude: Number;

        // The altitude in meters.
        public var altitude: Number;

        // The horizontal accuracy in meters.
        public var horizontalAccuracy: Number;

        // The vertical accuracy in meters.
        public var verticalAccuracy: Number;

        // The speed in meters/second.
        public var speed: Number;

        // The direction of movement (with respect to true north) in integer degrees.
        public var heading: Number;

        // The number of milliseconds at the time of the event since the runtime was initialized.
        // The timestamp is relative! The very first event contains a timestamp of 0.
        // Then it increases as the time pasts.
        public var timestamp: Number;

        public function GeolocationEvent(type:String = GeolocationEvent.UPDATE, bubbles:Boolean = false, cancelable:Boolean = false, latitude:Number = 0, longitude:Number = 0, altitude:Number = 0, horizontalAccuracy:Number = 0, verticalAccuracy:Number = 0, speed:Number = 0, heading:Number = 0, timestamp:Number = 0)
        {
            super(type,bubbles,cancelable);

            // Some values coming from the backend might be NaN.
            // But this event only accepts NaN in `altitude` and `heading` fields.
            // The other fields are set to 0 if the incoming values are NaN.
            this.altitude = altitude;
            this.heading = heading;

            if(isNaN(latitude)) {
                this.latitude = 0;
            } else {
                this.latitude = latitude;
            }
            if(isNaN(longitude)) {
                this.longitude = 0;
            } else {
                this.longitude = longitude;
            }
            if(isNaN(horizontalAccuracy)) {
                this.horizontalAccuracy = 0;
            } else {
                this.horizontalAccuracy = horizontalAccuracy;
            }
            if(isNaN(verticalAccuracy)) {
                this.verticalAccuracy = 0;
            } else {
                this.verticalAccuracy = verticalAccuracy;
            }
            if(isNaN(speed)) {
                this.speed = 0;
            } else {
                this.speed = speed;
            }
            if(isNaN(timestamp)) {
                this.timestamp = 0;
            } else {
                this.timestamp = timestamp;
            }
        }

        // [override] Creates a copy of the GeolocationEvent object and sets the value of each property to match that of the original.
        override public function clone():Event
        {
            return new GeolocationEvent(this.type,this.bubbles,this.cancelable,this.latitude,this.longitude,this.altitude,this.horizontalAccuracy,this.verticalAccuracy,this.speed,this.heading,this.timestamp);
        }

        // [override] Returns a string that contains all the properties of the GeolocationEvent object.
        override public function toString():String
        {
            return this.formatToString("GeolocationEvent","type","bubbles","cancelable","eventPhase","latitude","longitude","altitude","horizontalAccuracy","verticalAccuracy","speed","heading","timestamp");
        }
    }
}
