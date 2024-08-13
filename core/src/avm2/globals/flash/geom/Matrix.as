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

		public function clone():Matrix {
			return new Matrix(this.a,this.b,this.c,this.d,this.tx,this.ty);
		}

		public function concat(m:Matrix):void {
			var a = m.a * this.a + m.c * this.b;
			var b = m.b * this.a + m.d * this.b;
			var c = m.a * this.c + m.c * this.d;
			var d = m.b * this.c + m.d * this.d;
			var tx = m.a * this.tx + m.c * this.ty + m.tx;
			var ty = m.b * this.tx + m.d * this.ty + m.ty;

			this.a = a;
			this.b = b;
			this.c = c;
			this.d = d;
			this.tx = tx;
			this.ty = ty;
		}

		[API("674")]
		public function copyColumnFrom(column:uint, vector3D:Vector3D):void {
			// FP BUG: For some reason these methods are identical
			this.copyRowFrom(column, vector3D);
		}

		[API("674")]
		public function copyColumnTo(column:uint, vector3D:Vector3D):void {
			if(column == 0) {
				vector3D.x = this.a;
				vector3D.y = this.b;
				vector3D.z = 0;
			}
			else if (column == 1) {
				vector3D.x = this.c;
				vector3D.y = this.d;
				vector3D.z = 0;
			}
			else if (column == 2) {
				vector3D.x = this.tx;
				vector3D.y = this.ty;
				vector3D.z = 1;
			} // otherwise vector is unchanged
		}

		[API("674")]
		public function copyFrom(sourceMatrix: Matrix): void {
			this.a = sourceMatrix.a;
			this.b = sourceMatrix.b;
			this.c = sourceMatrix.c;
			this.d = sourceMatrix.d;
			this.tx = sourceMatrix.tx;
			this.ty = sourceMatrix.ty;
		}

		[API("674")]
		public function copyRowFrom(row: uint, vector3D: Vector3D): void {
			if (row == 0) {
				this.a = vector3D.x;
				this.c = vector3D.y;
				this.tx = vector3D.z;
			} else if(row == 1) {
				this.b = vector3D.x;
				this.d = vector3D.y;
				this.ty = vector3D.z;
			} // otherwise matrix is unchanged
		}

		[API("674")]
		public function copyRowTo(row:uint, vector3D:Vector3D):void {
			if(row == 0) {
				vector3D.x = this.a;
				vector3D.y = this.c;
				vector3D.z = this.tx;
			}
			else if (row == 1) {
				vector3D.x = this.b;
				vector3D.y = this.d;
				vector3D.z = this.ty;
			}
			else if (row == 2) {
				vector3D.x = 0;
				vector3D.y = 0;
				vector3D.z = 1;
			} // otherwise vector is unchanged
		}

		public function createBox(scaleX:Number, scaleY:Number, rotation:Number = 0, tx:Number = 0, ty:Number = 0):void {
			this.identity();
			this.rotate(rotation);
			this.scale(scaleX, scaleY);
			this.translate(tx, ty);
		}

		public function createGradientBox(width: Number, height: Number, rotation: Number = 0, tx: Number = 0, ty: Number = 0): void {
			this.createBox(width / 1638.4, height / 1638.4, rotation, tx + width / 2, ty + height / 2);
		}

		public function deltaTransformPoint(point:Point):Point {
			return new Point(this.a * point.x + this.c * point.y, this.b * point.x + this.d * point.y);
		}

		public function identity():void {
			this.a = 1;
			this.b = 0;
			this.c = 0;
			this.d = 1;
			this.tx = 0;
			this.ty = 0;
		}

		public function invert():void {
			var det = this.a * this.d - this.c * this.b;
			var tx = (this.d * this.tx - this.c * this.ty) / -det;
			var ty = (this.b * this.tx - this.a * this.ty) / det;
			var a = this.d / det;
			var b = this.b / -det;
			var c = this.c / -det;
			var d = this.a / det;

			this.a = a;
			this.b = b;
			this.c = c;
			this.d = d;
			this.tx = tx;
			this.ty = ty;
		}

		public function rotate(angle:Number):void {
			var sin = Math.sin(angle);
			var cos = Math.cos(angle);

			var a = cos * this.a + (-sin) * this.b;
			var b = sin * this.a + cos * this.b;
			var c = cos * this.c + (-sin) * this.d;
			var d = sin * this.c + cos * this.d;
			var tx = cos * this.tx + (-sin) * this.ty;
			var ty = sin * this.tx + cos * this.ty;

			this.a = a;
			this.b = b;
			this.c = c;
			this.d = d;
			this.tx = tx;
			this.ty = ty;
		}

		public function scale(sx:Number, sy:Number):void {
			this.a *= sx;
			this.b *= sy;
			this.c *= sx;
			this.d *= sy;
			this.tx *= sx;
			this.ty *= sy;
		}

		[API("674")]
		public function setTo(aa:Number, ba:Number, ca:Number, da:Number, txa:Number, tya:Number):void {
			this.a = aa;
			this.b = ba;
			this.c = ca;
			this.d = da;
			this.tx = txa;
			this.ty = tya;
		}

		public function toString():String {
			return "(a=" + this.a + ", b=" + this.b + ", c=" + this.c + ", d=" + this.d + ", tx=" + this.tx + ", ty=" + this.ty + ")";
		}

		public function transformPoint(point:Point):Point {
			return new Point(this.a * point.x + this.c * point.y + this.tx, this.b * point.x + this.d * point.y + this.ty);
		}

		public function translate(dx:Number, dy:Number):void {
			this.tx += dx;
			this.ty += dy;
		}
	}
}
