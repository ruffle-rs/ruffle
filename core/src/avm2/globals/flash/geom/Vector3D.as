package flash.geom {
    public class Vector3D {

        // `describeType` returns these in this weird order
        public static const Z_AXIS : Vector3D = new Vector3D(0, 0, 1);
        public static const X_AXIS : Vector3D = new Vector3D(1, 0, 0);
        public static const Y_AXIS : Vector3D = new Vector3D(0, 1, 0);

        public static function angleBetween(a:Vector3D, b:Vector3D):Number {
            return Math.acos(a.dotProduct(b) / (a.length * b.length));
        }

        public static function distance(pt1:Vector3D, pt2:Vector3D):Number {
            return pt2.subtract(pt1).length;
        }

        public function Vector3D(x:Number = 0, y:Number = 0, z:Number = 0, w:Number = 0) {
            this.x = x;
            this.y = y;
            this.z = z;
            this.w = w;
        }

        public var w:Number;
        public var x:Number;
        public var y:Number;
        public var z:Number;

        public function get length():Number {
            return Math.sqrt(this.lengthSquared);
        }

        public function get lengthSquared():Number {
            return this.x * this.x + this.y * this.y + this.z * this.z;
        }

        public function toString():String {
            return "Vector3D(" + this.x + ", " + this.y + ", " + this.z + ")";
        }

        public function add(a:Vector3D):Vector3D {
            // w is ignored
            return new Vector3D(this.x + a.x, this.y + a.y, this.z + a.z);
        }

        public function subtract(a:Vector3D):Vector3D {
            // w is ignored
            return new Vector3D(this.x - a.x, this.y - a.y, this.z - a.z);
        }

        public function incrementBy(a:Vector3D):void {
            this.x += a.x;
            this.y += a.y;
            this.z += a.z;
            // w is unchanged
        }

        public function decrementBy(a:Vector3D):void {
            this.x -= a.x;
            this.y -= a.y;
            this.z -= a.z;
            // w is unchanged
        }

        public function clone():Vector3D {
            return new Vector3D(this.x, this.y, this.z, this.w);
        }

        [API("674")]
        public function copyFrom(sourceVector3D:Vector3D):void {
            this.x = sourceVector3D.x;
            this.y = sourceVector3D.y;
            this.z = sourceVector3D.z;
            // w is unchanged
        }

        public function equals(toCompare:Vector3D, allFour:Boolean = false):Boolean {
            return this.x == toCompare.x && this.y == toCompare.y && this.z == toCompare.z && (!allFour || this.w == toCompare.w);
        }

        public function nearEquals(toCompare:Vector3D, tolerance:Number, allFour:Boolean = false):Boolean {
            // Looks like there is a Flash Player bug here:
            // With allFour=true, this.w is ignored, only toCompare.w is compared
            // with tolerance ... I think they forgot to do the subtraction there.
            return (Math.abs(this.x - toCompare.x) < tolerance)
                && (Math.abs(this.y - toCompare.y) < tolerance)
                && (Math.abs(this.z - toCompare.z) < tolerance)
                && (!allFour || Math.abs(toCompare.w) < tolerance); // FP BUG
        }

        [API("674")]
        public function setTo(xa:Number, ya:Number, za: Number):void {
            this.x = xa;
            this.y = ya;
            this.z = za;
            // w is unchanged
        }

        public function scaleBy(s:Number):void {
            this.x *= s;
            this.y *= s;
            this.z *= s;
            // w is unchanged
        }

        public function negate():void {
            this.x *= -1;
            this.y *= -1;
            this.z *= -1;
            // w is unchanged
        }

        public function project():void {
            this.x /= this.w;
            this.y /= this.w;
            this.z /= this.w;
            // w is unchanged
        }

        public function normalize():Number {
            var len : Number = this.length;

            if (len == 0) {
                this.x = 0;
                this.y = 0;
                this.z = 0;
            }
            else if (len > 0) {
                this.x /= len;
                this.y /= len;
                this.z /= len;
            }
            else { // if len (so any of the components) is NaN or undefined
                this.x = NaN;
                this.y = NaN;
                this.z = NaN;
            }

            return len; // returns the original length
        }

        public function dotProduct(a:Vector3D):Number {
            return this.x * a.x + this.y * a.y + this.z * a.z;
        }

        public function crossProduct(a:Vector3D):Vector3D {
            return new Vector3D(
                this.y * a.z - this.z * a.y,
                this.z * a.x - this.x * a.z,
                this.x * a.y - this.y * a.x,
                1); // for whatever reason w is always set to 1
        }
    }
}
