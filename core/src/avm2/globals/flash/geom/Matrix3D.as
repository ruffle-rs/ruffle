// Based on the MIT-licensed OpenFL code https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx

package flash.geom {
	public class Matrix3D {

		// The 4x4 matrix data, stored in column-major order
		private var _rawData:Vector.<Number>;

		public function get rawData():Vector.<Number> {
			return this._rawData.concat();
		}

		public function set rawData(value:Vector.<Number>):void {
			this._rawData = value.concat();
		}

		public function Matrix3D(v:Vector.<Number> = null) {
			this._rawData = v;
			if (this._rawData == null) {
				this.identity();
			}
		}

		public function identity():void {
			// Note that every 4 elements is a *column*, not a row
			this._rawData = new <Number>[
					1, 0, 0, 0,
					0, 1, 0, 0,
					0, 0, 1, 0,
					0, 0, 0, 1
				];
		}

		public function appendTranslation(x:Number, y:Number, z:Number):void {
			this._rawData[12] += x;
			this._rawData[13] += y;
			this._rawData[14] += z;
		}

		public function appendRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void {
			var tx:Number, ty:Number, tz:Number;
			tx = ty = tz = 0;

			if (pivotPoint != null) {
				tx = pivotPoint.x;
				ty = pivotPoint.y;
				tz = pivotPoint.z;
			}
			var radian = degrees * Math.PI / 180;
			var cos = Math.cos(radian);
			var sin = Math.sin(radian);
			var x = axis.x;
			var y = axis.y;
			var z = axis.z;
			var x2 = x * x;
			var y2 = y * y;
			var z2 = z * z;
			var ls = x2 + y2 + z2;
			if (ls != 0) {
				var l = Math.sqrt(ls);
				x /= l;
				y /= l;
				z /= l;
				x2 /= ls;
				y2 /= ls;
				z2 /= ls;
			}
			var ccos = 1 - cos;
			var m = new Matrix3D();

			var d = m.rawData;
			d[0] = x2 + (y2 + z2) * cos;
			d[1] = x * y * ccos + z * sin;
			d[2] = x * z * ccos - y * sin;
			d[4] = x * y * ccos - z * sin;
			d[5] = y2 + (x2 + z2) * cos;
			d[6] = y * z * ccos + x * sin;
			d[8] = x * z * ccos + y * sin;
			d[9] = y * z * ccos - x * sin;
			d[10] = z2 + (x2 + y2) * cos;
			d[12] = (tx * (y2 + z2) - x * (ty * y + tz * z)) * ccos + (ty * z - tz * y) * sin;
			d[13] = (ty * (x2 + z2) - y * (tx * x + tz * z)) * ccos + (tz * x - tx * z) * sin;
			d[14] = (tz * (x2 + y2) - z * (tx * x + ty * y)) * ccos + (tx * y - ty * x) * sin;
			m.rawData = d;

			this.append(m);
		}

		public function copyRawDataFrom(vector:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void {
			if (transpose) {
				this.transpose();
			}

			var length = vector.length - index;

			for (var i = 0; i < length; i++) {
				this._rawData[i] = vector[i + index];
			}

			if (transpose) {
				this.transpose();
			}
		}

		public function transpose():void {
			// Make a copy
			var oRawData = this._rawData.concat();
			this._rawData[1] = oRawData[4];
			this._rawData[2] = oRawData[8];
			this._rawData[3] = oRawData[12];
			this._rawData[4] = oRawData[1];
			this._rawData[6] = oRawData[9];
			this._rawData[7] = oRawData[13];
			this._rawData[8] = oRawData[2];
			this._rawData[9] = oRawData[6];
			this._rawData[11] = oRawData[14];
			this._rawData[12] = oRawData[3];
			this._rawData[13] = oRawData[7];
			this._rawData[14] = oRawData[11];
		}
		public function append(lhs:Matrix3D):void {
			var m111:Number = this._rawData[0],
				m121:Number = this._rawData[4],
				m131:Number = this._rawData[8],
				m141:Number = this._rawData[12],
				m112:Number = this._rawData[1],
				m122:Number = this._rawData[5],
				m132:Number = this._rawData[9],
				m142:Number = this._rawData[13],
				m113:Number = this._rawData[2],
				m123:Number = this._rawData[6],
				m133:Number = this._rawData[10],
				m143:Number = this._rawData[14],
				m114:Number = this._rawData[3],
				m124:Number = this._rawData[7],
				m134:Number = this._rawData[11],
				m144:Number = this._rawData[15],
				m211:Number = lhs._rawData[0],
				m221:Number = lhs._rawData[4],
				m231:Number = lhs._rawData[8],
				m241:Number = lhs._rawData[12],
				m212:Number = lhs._rawData[1],
				m222:Number = lhs._rawData[5],
				m232:Number = lhs._rawData[9],
				m242:Number = lhs._rawData[13],
				m213:Number = lhs._rawData[2],
				m223:Number = lhs._rawData[6],
				m233:Number = lhs._rawData[10],
				m243:Number = lhs._rawData[14],
				m214:Number = lhs._rawData[3],
				m224:Number = lhs._rawData[7],
				m234:Number = lhs._rawData[11],
				m244:Number = lhs._rawData[15];

			this._rawData[0] = m111 * m211 + m112 * m221 + m113 * m231 + m114 * m241;
			this._rawData[1] = m111 * m212 + m112 * m222 + m113 * m232 + m114 * m242;
			this._rawData[2] = m111 * m213 + m112 * m223 + m113 * m233 + m114 * m243;
			this._rawData[3] = m111 * m214 + m112 * m224 + m113 * m234 + m114 * m244;

			this._rawData[4] = m121 * m211 + m122 * m221 + m123 * m231 + m124 * m241;
			this._rawData[5] = m121 * m212 + m122 * m222 + m123 * m232 + m124 * m242;
			this._rawData[6] = m121 * m213 + m122 * m223 + m123 * m233 + m124 * m243;
			this._rawData[7] = m121 * m214 + m122 * m224 + m123 * m234 + m124 * m244;

			this._rawData[8] = m131 * m211 + m132 * m221 + m133 * m231 + m134 * m241;
			this._rawData[9] = m131 * m212 + m132 * m222 + m133 * m232 + m134 * m242;
			this._rawData[10] = m131 * m213 + m132 * m223 + m133 * m233 + m134 * m243;
			this._rawData[11] = m131 * m214 + m132 * m224 + m133 * m234 + m134 * m244;

			this._rawData[12] = m141 * m211 + m142 * m221 + m143 * m231 + m144 * m241;
			this._rawData[13] = m141 * m212 + m142 * m222 + m143 * m232 + m144 * m242;
			this._rawData[14] = m141 * m213 + m142 * m223 + m143 * m233 + m144 * m243;
			this._rawData[15] = m141 * m214 + m142 * m224 + m143 * m234 + m144 * m244;
		}

		// Based on https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L307
		public function appendScale(xScale:Number, yScale:Number, zScale:Number):void {
			this.append(new Matrix3D(Vector.<Number>([
					xScale, 0.0, 0.0, 0.0, 0.0, yScale, 0.0, 0.0, 0.0, 0.0, zScale, 0.0, 0.0, 0.0, 0.0, 1.0
				])));
		}

		public function prependTranslation(x:Number, y:Number, z:Number):void {
			var m = new Matrix3D();
			m.position = new Vector3D(x, y, z);
			this.prepend(m);
		}

		public function get position():Vector3D {
			return new Vector3D(_rawData[12], _rawData[13], _rawData[14]);
		}

		public function set position(val:Vector3D):void {
			this._rawData[12] = val.x;
			this._rawData[13] = val.y;
			this._rawData[14] = val.z;
		}

		public function prepend(rhs:Matrix3D):void {
			var m111:Number = rhs._rawData[0],
				m121:Number = rhs._rawData[4],
				m131:Number = rhs._rawData[8],
				m141:Number = rhs._rawData[12],
				m112:Number = rhs._rawData[1],
				m122:Number = rhs._rawData[5],
				m132:Number = rhs._rawData[9],
				m142:Number = rhs._rawData[13],
				m113:Number = rhs._rawData[2],
				m123:Number = rhs._rawData[6],
				m133:Number = rhs._rawData[10],
				m143:Number = rhs._rawData[14],
				m114:Number = rhs._rawData[3],
				m124:Number = rhs._rawData[7],
				m134:Number = rhs._rawData[11],
				m144:Number = rhs._rawData[15],
				m211:Number = this._rawData[0],
				m221:Number = this._rawData[4],
				m231:Number = this._rawData[8],
				m241:Number = this._rawData[12],
				m212:Number = this._rawData[1],
				m222:Number = this._rawData[5],
				m232:Number = this._rawData[9],
				m242:Number = this._rawData[13],
				m213:Number = this._rawData[2],
				m223:Number = this._rawData[6],
				m233:Number = this._rawData[10],
				m243:Number = this._rawData[14],
				m214:Number = this._rawData[3],
				m224:Number = this._rawData[7],
				m234:Number = this._rawData[11],
				m244:Number = this._rawData[15];

			this._rawData[0] = m111 * m211 + m112 * m221 + m113 * m231 + m114 * m241;
			this._rawData[1] = m111 * m212 + m112 * m222 + m113 * m232 + m114 * m242;
			this._rawData[2] = m111 * m213 + m112 * m223 + m113 * m233 + m114 * m243;
			this._rawData[3] = m111 * m214 + m112 * m224 + m113 * m234 + m114 * m244;

			this._rawData[4] = m121 * m211 + m122 * m221 + m123 * m231 + m124 * m241;
			this._rawData[5] = m121 * m212 + m122 * m222 + m123 * m232 + m124 * m242;
			this._rawData[6] = m121 * m213 + m122 * m223 + m123 * m233 + m124 * m243;
			this._rawData[7] = m121 * m214 + m122 * m224 + m123 * m234 + m124 * m244;

			this._rawData[8] = m131 * m211 + m132 * m221 + m133 * m231 + m134 * m241;
			this._rawData[9] = m131 * m212 + m132 * m222 + m133 * m232 + m134 * m242;
			this._rawData[10] = m131 * m213 + m132 * m223 + m133 * m233 + m134 * m243;
			this._rawData[11] = m131 * m214 + m132 * m224 + m133 * m234 + m134 * m244;

			this._rawData[12] = m141 * m211 + m142 * m221 + m143 * m231 + m144 * m241;
			this._rawData[13] = m141 * m212 + m142 * m222 + m143 * m232 + m144 * m242;
			this._rawData[14] = m141 * m213 + m142 * m223 + m143 * m233 + m144 * m243;
			this._rawData[15] = m141 * m214 + m142 * m224 + m143 * m234 + m144 * m244;
		}

		public function copyFrom(other:Matrix3D):void {
			// This makes a copy of other.rawData
			this._rawData = other.rawData;
		}

		public function copyRawDataTo(vector:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void {
			if (transpose) {
				this.transpose();
			}

			for (var i = 0; i < rawData.length; i++) {
				vector[i + index] = _rawData[i];
			}

			if (transpose) {
				this.transpose();
			}
		}

		public function clone():Matrix3D {
			return new Matrix3D(this.rawData);
		}

	}
}