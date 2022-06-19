package flash.geom {
	public class Matrix {
		public var a:Number;
		public var b:Number;
		public var c:Number;
		public var d:Number;
		public var tx:Number;
		public var ty:Number;

		public function Matrix(a:Number = 1, b:Number = 0, c:Number = 0, d:Number = 1, tx:Number = 0, ty:Number = 0) {
			this.a = a;
			this.b = b;
			this.c = c;
			this.d = d;
			this.tx = tx;
			this.ty = ty;
		}

		public function identity():void {
			this.a = 1;
			this.b = 0;
			this.c = 0;
			this.d = 1;
			this.tx = 0;
			this.ty = 0;
		}

		public function scale(sx:Number, sy:Number):void {
			this.a *= sx;
			this.d *= sy;
		}

		public function toString():String {
			return "(a=" + this.a + ", b=" + this.b + ", c=" + this.c + ", d=" + this.d + ", tx=" + this.tx + ", ty=" + this.ty + ")";
		}
	}
}
