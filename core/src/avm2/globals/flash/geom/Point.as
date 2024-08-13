package flash.geom {
	public class Point {
		public var x:Number;
		public var y:Number;

		public function Point(x:Number = 0, y:Number = 0) {
			this.x = x;
			this.y = y;
		}

		public function get length():Number {
			return Math.sqrt(this.x * this.x + this.y * this.y);
		}

		public function toString():String {
			return "(x=" + this.x + ", y=" + this.y + ")";
		}

		public function add(v:Point):Point {
			return new Point(this.x + v.x, this.y + v.y);	
		}

		public function subtract(v:Point):Point {
			return new Point(this.x - v.x, this.y - v.y);	
		}

		public function clone():Point {
			return new Point(this.x, this.y);
		}

		[API("674")]
		public function copyFrom(sourcePoint:Point):void {
			this.x = sourcePoint.x;
			this.y = sourcePoint.y;
		}

		public function equals(toCompare:Point):Boolean {
			return this.x == toCompare.x && this.y == toCompare.y;
		}

		public function normalize(thickness:Number):void {
			var len:Number = this.length;
			if (len > 0) {
				var inv_d:Number = thickness / len;
				this.x *= inv_d;
				this.y *= inv_d;
			}	
		}

		public function offset(dx:Number, dy:Number):void {
			this.x += dx;
			this.y += dy;
		}

		[API("674")]
		public function setTo(xa:Number, ya:Number):void {
			this.x = xa;
			this.y = ya;
		}

		public static function distance(pt1:Point, pt2:Point):Number {
			return pt2.subtract(pt1).length;
		}

		public static function interpolate(pt1:Point, pt2:Point, f:Number):Point {
			return new Point(pt2.x - (pt2.x - pt1.x) * f, pt2.y - (pt2.y - pt1.y) * f);
		}

		public static function polar(len:Number, angle:Number):Point {
			return new Point(len* Math.cos(angle), len * Math.sin(angle));
		}
	}
}
