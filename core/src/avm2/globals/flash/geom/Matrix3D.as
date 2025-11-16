/**
 * The Matrix3D class represents a transformation matrix that determines the position and
 * orientation of a three-dimensional (3D) display object. The matrix can perform
 * transformation functions including translation (repositioning along the x, y, and z
 * axes), rotation, and scaling (resizing). The Matrix3D class can also perform
 * perspective projection, which maps points from the 3D coordinate space to a
 * two-dimensional (2D) view.
 *
 * A single matrix can combine multiple transformations and apply them at once to a 3D
 * display object. For example, a matrix can be applied to 3D coordinates to perform a
 * rotation followed by a translation.
 *
 * When you explicitly set the `z` property or any of the rotation or scaling properties
 * of a display object, a corresponding Matrix3D object is automatically created.
 *
 * You can access a 3D display object's Matrix3D object through the `transform.matrix3d`
 * property. 2D objects do not have a Matrix3D object.
 *
 * The value of the `z` property of a 2D object is zero and the value of its `matrix3D`
 * property is `null`.
 *
 * **Note:** If the same Matrix3D object is assigned to two different display objects, a
 * runtime error is thrown.
 *
 * The Matrix3D class uses a 4x4 square matrix: a table of four rows and columns of
 * numbers that hold the data for the transformation. The first three rows of the matrix
 * hold data for each 3D axis (x,y,z). The translation information is in the last column.
 * The orientation and scaling data are in the first three columns. The scaling factors
 * are the diagonal numbers in the first three columns. Here is a representation of
 * Matrix3D elements:
 *
 *   scalex        shearYX       shearZX       perspectiveX
 *   shearXY       scaleY        shearZY       perspectiveY
 *   shearXZ       shearYZ       scaleZ        perspectiveZ
 *   transitionx   transitionY   transitionZ   perspectiveS
 *
 * You don't need to understand matrix mathematics to use the Matrix3D class. It offers
 * specific methods that simplify the task of transformation and projection, such as the
 * `appendTranslation()`, `appendRotation()`, or `interpolateTo()` methods. You also can
 * use the `decompose()` and `recompose()` methods or the `rawData` property to access
 * the underlying matrix elements.
 *
 * Display objects cache their axis rotation properties to have separate rotation for
 * each axis and to manage the different combinations of rotations. When a method of a
 * Matrix3D object is called to transform a display object, the rotation cache of the
 * object is invalidated.
 */
// Based on the MIT-licensed OpenFL code https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx

package flash.geom {

    import flash.geom.Orientation3D;

    public class Matrix3D {

        private static var _correct:int = 0; // Apply scale skew perspective to interpolate methods
                                             // I would recommend setting this to 1 by default

        /**
         * Gets or sets the value of the 'correct' property.
         * This property is used to apply scale, skew, and perspective to interpolate methods.
         * You can call it globally with Matrix3d.correct = true; to force correct behaviour
         * 
         * @param value Boolean - The value to set for the 'correct' property. 
         *                        If true, the state is considered correct; if false, incorrect.
         * @return Boolean - Returns true if the state is correct (non-zero), otherwise false.
         */
        public static function get correct():Boolean {
            return _correct != 0;
        }

        public static function set correct(value:Boolean):void {
            _correct = value ? 1 : 0;
        }

        /**
         * A Vector of 16 Numbers, where every four elements is a column of a 4x4 matrix.
         * 
         * An exception is thrown if the `rawData` property is set to a matrix that is not
         * invertible. The Matrix3D object must be invertible. If a non-invertible matrix is
         * needed, create a subclass of the Matrix3D object.
         */
        // The 4x4 matrix data, stored in column-major order
        // This is never null.
        [Ruffle(NativeAccessible)]
        private var _rawData:Vector.<Number>;

        public function get rawData():Vector.<Number> {
            return this._rawData.AS3::concat();
        }

        public function set rawData(value:Vector.<Number>):void {
            if (value != null) {
                this._rawData = value.AS3::concat();
            }
        }

        public function Matrix3D(v:Vector.<Number> = null) {
            if (v != null && v.length == 16) {
                //this._rawData = v.AS3::concat();
                this.rawData = v; // use setter... must be tested if much slower...
            }
            else {
                this.identity();
            }
        }

        /**
         * Converts the current matrix to an identity or unit matrix. An identity matrix has
         * a value of one for the elements on the main diagonal and a value of zero for all
         * other elements. The result is a matrix where the rawData value is
         * 1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1 and the rotation setting is set to
         * `Vector3D(0,0,0)`, the position or translation setting is set to `Vector3D(0,0,0)`,
         * and the scale is set to `Vector3D(1,1,1)`. Here is a representation of an
         * identity matrix.
         * 
         *   1, 0, 0, 0,
         *   0, 1, 0, 0,
         *   0, 0, 1, 0,
         *   0, 0, 0, 1
         * 
         * An object transformed by applying an identity matrix performs no transformation.
         * In other words, if a matrix is multiplied by an identity matrix, the result is a
         * matrix that is the same as (identical to) the original matrix.
         */
        public function identity():void {
            // Note that every 4 elements is a *column*, not a row
            this._rawData = new <Number>[
                    1, 0, 0, 0,
                    0, 1, 0, 0,
                    0, 0, 1, 0,
                    0, 0, 0, 1
                ];
        }

        /**
         * Appends an incremental translation, a repositioning along the x, y, and z axes,
         * to a Matrix3D object. When the Matrix3D object is applied to a display object,
         * the matrix performs the translation changes after other transformations in the
         * Matrix3D object.
         * 
         * The translation is defined as a set of three incremental changes along the
         * three axes (x,y,z). When the transformation is applied to a display object, the
         * display object moves from it current location along the x, y, and z axes as
         * specified by the parameters. To make sure that the translation only affects a
         * specific axis, set the other parameters to zero. A zero parameter means no change
         * along the specific axis.
         * 
         * The translation changes are not absolute. They are relative to the current
         * position and orientation of the matrix. To make an absolute change to the
         * transformation matrix, use the recompose() method. The order of transformation
         * also matters. A translation followed by a rotation transformation produces a
         * different effect than a rotation followed by a translation.
         * 
         * @param   x   An incremental translation along the x axis.
         * @param   y   An incremental translation along the y axis.
         * @param   z   An incremental translation along the z axis.
         */
        public function appendTranslation(x:Number, y:Number, z:Number):void {
            this._rawData[12] += x;
            this._rawData[13] += y;
            this._rawData[14] += z;
        }

        /**
         * Appends an incremental rotation to a Matrix3D object. When the Matrix3D object is
         * applied to a display object, the matrix performs the rotation after other
         * transformations in the Matrix3D object.
         * 
         * The display object's rotation is defined by an axis, an incremental degree of
         * rotation around the axis, and an optional pivot point for the center of the
         * object's rotation. The axis can be any general direction. The common axes are the
         * `X_AXIS (Vector3D(1,0,0))`, `Y_AXIS (Vector3D(0,1,0))`, and
         * `Z_AXIS (Vector3D(0,0,1))`. In aviation terminology, the rotation about the y axis
         * is called yaw. The rotation about the x axis is called pitch. The rotation about
         * the z axis is called roll.
         * 
         * The order of transformation matters. A rotation followed by a translation
         * transformation produces a different effect than a translation followed by a
         * rotation transformation.
         * 
         * The rotation effect is not absolute. It is relative to the current position and
         * orientation. To make an absolute change to the transformation matrix, use the
         * `recompose()` method. The `appendRotation()` method is also different from the
         * axis rotation property of the display object, such as `rotationX` property. The
         * `rotation` property is always performed before any translation, whereas the
         * `appendRotation()` method is performed relative to what is already in the matrix.
         * To make sure that you get a similar effect as the display object's axis rotation
         * property, use the `prependRotation()` method, which performs the rotation before
         * other transformations in the matrix.
         * 
         * When the `appendRotation()` method's transformation is applied to a Matrix3D object
         * of a display object, the cached rotation property values of the display object
         * are invalidated.
         * 
         * One way to have a display object rotate around a specific point relative to its
         * location is to set the translation of the object to the specified point, rotate
         * the object using the `appendRotation()` method, and translate the object back to
         * the original position. In the following example, the myObject 3D display object
         * makes a y-axis rotation around the coordinate (10,10,0).
         * 
         * ```
         * myObject.z = 1;
         * myObject.transform.matrix3D.appendTranslation(10,10,0);
         * myObject.transform.matrix3D.appendRotation(1, Vector3D.Y_AXIS);
         * myObject.transform.matrix3D.appendTranslation(-10,-10,0);
         * ```
         * 
         * @param   degrees The degree of the rotation.
         * @param   axis    The axis or direction of rotation. The usual axes are the
         * `X_AXIS (Vector3D(1,0,0))`, `Y_AXIS (Vector3D(0,1,0))`, and
         * `Z_AXIS (Vector3D(0,0,1))`. This vector should have a length of one.
         * @param   pivotPoint  A point that determines the center of an object's rotation.
         * The default pivot point for an object is its registration point.
         */
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

            var mr = m.rawData;
            mr[0] = x2 + (y2 + z2) * cos;
            mr[1] = x * y * ccos + z * sin;
            mr[2] = x * z * ccos - y * sin;

            mr[4] = x * y * ccos - z * sin;
            mr[5] = y2 + (x2 + z2) * cos;
            mr[6] = y * z * ccos + x * sin;

            mr[8] = x * z * ccos + y * sin;
            mr[9] = y * z * ccos - x * sin;
            mr[10] = z2 + (x2 + y2) * cos;

            mr[12] = (tx * (y2 + z2) - x * (ty * y + tz * z)) * ccos + (ty * z - tz * y) * sin;
            mr[13] = (ty * (x2 + z2) - y * (tx * x + tz * z)) * ccos + (tz * x - tx * z) * sin;
            mr[14] = (tz * (x2 + y2) - z * (tx * x + ty * y)) * ccos + (tx * y - ty * x) * sin;
            m.rawData = mr;

            this.append(m);
        }

        /**
         * Copies all of the vector data from the source vector object into the calling
         * Matrix3D object. The optional `index` parameter allows you to select any starting
         * slot in the vector.
         * 
         * @param   vector  The vector object from which to copy the data.
         * @param   index
         * @param   transpose
         */
        [API("674")]
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

        /**
         * Copies specific row of the calling Matrix3D object into the Vector3D object.
         * 
         * @param   row The row from which to copy the data from.
         * @param   vector3D    The Vector3D object to copy the data into.
         */
        // Based on https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx#L542C1-L573
        [API("674")]
        public function copyRowTo(row:uint, vector3D:Vector3D):void {
            if (row > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            switch (row) {
                case 0:
                    vector3D.x = this._rawData[0];
                    vector3D.y = this._rawData[4];
                    vector3D.z = this._rawData[8];
                    vector3D.w = this._rawData[12];
                    break;
                case 1:
                    vector3D.x = this._rawData[1];
                    vector3D.y = this._rawData[5];
                    vector3D.z = this._rawData[9];
                    vector3D.w = this._rawData[13];
                    break;
                case 2:
                    vector3D.x = this._rawData[2];
                    vector3D.y = this._rawData[6];
                    vector3D.z = this._rawData[10];
                    vector3D.w = this._rawData[14];
                    break;
                case 3:
                    vector3D.x = this._rawData[3];
                    vector3D.y = this._rawData[7];
                    vector3D.z = this._rawData[11];
                    vector3D.w = this._rawData[15];
                    break;
            }
        }

        /**
         * Copies a Vector3D object into specific row of the calling Matrix3D object.
         * 
         * @param   row The row from which to copy the data to.
         * @param   vector3D    The Vector3D object from which to copy the data.
         */
        // Based on https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx#L504-L534
        [API("674")]
        public function copyRowFrom(row:uint, vector3D:Vector3D):void {
            if (row > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            
            switch (row) {
                case 0:
                    this._rawData[0] = vector3D.x;
                    this._rawData[4] = vector3D.y;
                    this._rawData[8] = vector3D.z;
                    this._rawData[12] = vector3D.w;
                    break;
                case 1:
                    this._rawData[1] = vector3D.x;
                    this._rawData[5] = vector3D.y;
                    this._rawData[9] = vector3D.z;
                    this._rawData[13] = vector3D.w;
                    break;
                case 2:
                    this._rawData[2] = vector3D.x;
                    this._rawData[6] = vector3D.y;
                    this._rawData[10] = vector3D.z;
                    this._rawData[14] = vector3D.w;
                    break;
                case 3:
                    this._rawData[3] = vector3D.x;
                    this._rawData[7] = vector3D.y;
                    this._rawData[11] = vector3D.z;
                    this._rawData[15] = vector3D.w;
                    break;
            }
        }

        /**
         * Uses the transformation matrix without its translation elements to transform a
         * Vector3D object from one space coordinate to another. The returned Vector3D
         * object holds the new coordinates after the rotation and scaling transformations
         * have been applied. If the `deltaTransformVector()` method applies a matrix that
         * only contains a translation transformation, the returned Vector3D is the same as
         * the original Vector3D object.
         * 
         * You can use the `deltaTransformVector()` method to have a display object in one
         * coordinate space respond to the rotation transformation of a second display
         * object. The object does not copy the rotation; it only changes its position to
         * reflect the changes in the rotation. For example, to use the display.Graphics
         * API for drawing a rotating 3D display object, you must map the object's rotating
         * coordinates to a 2D point. First, retrieve the object's 3D coordinates after each
         * rotation, using the `deltaTransformVector()` method. Next, apply the display
         * object's `local3DToGlobal()` method to translate the 3D coordinates to 2D points.
         * You can then use the 2D points to draw the rotating 3D object.
         * 
         * **Note:** This method automatically sets the `w` component of the passed Vector3D
         * to 0.0.
         * 
         * @param   v   A Vector3D object holding the coordinates that are going to be
         * transformed.
         * @return  Vector3D    A Vector3D object with the transformed coordinates.
         */
        public function deltaTransformVector(v:Vector3D):Vector3D {
            var x:Number = this._rawData[0] * v.x + this._rawData[4] * v.y + this._rawData[8] * v.z;
            var y:Number = this._rawData[1] * v.x + this._rawData[5] * v.y + this._rawData[9] * v.z;
            var z:Number = this._rawData[2] * v.x + this._rawData[6] * v.y + this._rawData[10] * v.z;
            var w:Number = this._rawData[3] * v.x + this._rawData[7] * v.y + this._rawData[11] * v.z;
            return new Vector3D(x, y, z, w);
        }

        /**
         * Uses the transformation matrix without its translation elements to transform a
         * Vector3D object from one space coordinate to another. The returned Vector3D
         * object holds the new coordinates after the rotation and scaling transformations
         * have been applied. If the `deltaTransformVector()` method applies a matrix that
         * only contains a translation transformation, the returned Vector3D is the same as
         * the original Vector3D object.
         * 
         * You can use the `deltaTransformVector()` method to have a display object in one
         * coordinate space respond to the rotation transformation of a second display
         * object. The object does not copy the rotation; it only changes its position to
         * reflect the changes in the rotation. For example, to use the display.Graphics
         * API for drawing a rotating 3D display object, you must map the object's rotating
         * coordinates to a 2D point. First, retrieve the object's 3D coordinates after each
         * rotation, using the `deltaTransformVector()` method. Next, apply the display
         * object's `local3DToGlobal()` method to translate the 3D coordinates to 2D points.
         * You can then use the 2D points to draw the rotating 3D object.
         * 
         * **Note:** This method automatically sets the `w` component of the passed Vector3D
         * to 0.0.
         * 
         * @param   v   A Vector3D object holding the coordinates that are going to be
         * transformed.
         * @return  Vector3D    A Vector3D object with the transformed coordinates.
         */
        public function transformVector(v:Vector3D):Vector3D {
            var x:Number = this._rawData[0] * v.x + this._rawData[4] * v.y + this._rawData[8] * v.z + this._rawData[12];
            var y:Number = this._rawData[1] * v.x + this._rawData[5] * v.y + this._rawData[9] * v.z + this._rawData[13];
            var z:Number = this._rawData[2] * v.x + this._rawData[6] * v.y + this._rawData[10] * v.z + this._rawData[14];
            var w:Number = this._rawData[3] * v.x + this._rawData[7] * v.y + this._rawData[11] * v.z + this._rawData[15];
            return new Vector3D(x, y, z, w);
        }

        public function transformVectors(vin:Vector.<Number>, vout:Vector.<Number>):void {
            if (vin == null) {
                throw new TypeError("Error #2007: Parameter vin must be non-null.", 2007);
            }
            if (vout == null) {
                throw new TypeError("Error #2007: Parameter vout must be non-null.", 2007);
            }

            var resultVecsLength:Number = Math.floor(vin.length / 3) * 3;
            if (resultVecsLength > vout.length && vout.fixed) {
                throw new RangeError("Error #1126: Cannot change the length of a fixed Vector.")
            }

            var result3D:Vector3D;
            for (var i = 0; i < resultVecsLength; i += 3) {
                result3D = transformVector(new Vector3D(vin[i], vin[i + 1], vin[i + 2]));
                if (i <= vout.length) {
                    vout[i] = result3D.x;
                    vout[i + 1] = result3D.y;
                    vout[i + 2] = result3D.z;
                } else {
                    vout.push(result3D.x, result3D.y, result3D.z);
                }
            }
        }

        /**
         * Converts the current Matrix3D object to a matrix where the rows and columns are
         * swapped. For example, if the current Matrix3D object's rawData contains the
         * following 16 numbers, `1,2,3,4,11,12,13,14,21,22,23,24,31,32,33,34`, the
         * `transpose()` method reads every four elements as a row and turns the rows into
         * columns. The result is a matrix with the rawData of:
         * `1,11,21,31,2,12,22,32,3,13,23,33,4,14,24,34`.
         * 
         * The `transpose()` method replaces the current matrix with a transposed matrix.
         * If you want to transpose a matrix without altering the current matrix, first copy
         * the current matrix by using the `clone()` method and then apply the `transpose()`
         * method to the copy.
         * 
         * An orthogonal matrix is a square matrix whose transpose is equal to its inverse.
         */
        [Ruffle(NativeCallable)]
        public function transpose():void {
            // Make a copy
            var mr = this.rawData;
            this._rawData[1] = mr[4];
            this._rawData[2] = mr[8];
            this._rawData[3] = mr[12];
            this._rawData[4] = mr[1];
            this._rawData[6] = mr[9];
            this._rawData[7] = mr[13];
            this._rawData[8] = mr[2];
            this._rawData[9] = mr[6];
            this._rawData[11] = mr[14];
            this._rawData[12] = mr[3];
            this._rawData[13] = mr[7];
            this._rawData[14] = mr[11];
        }

        /**
         * A Vector of 16 Numbers, where every four elements is a column of a 4x4 matrix.
         * 
         * An exception is thrown if the `rawData` property is set to a matrix that is not
         * invertible. The Matrix3D object must be invertible. If a non-invertible matrix is
         * needed, create a subclass of the Matrix3D object.
         */
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

        /**
         * Appends an incremental scale change along the x, y, and z axes to a Matrix3D
         * object. When the Matrix3D object is applied to a display object, the matrix
         * performs the scale changes after other transformations in the Matrix3D object.
         * The default scale factor is (1.0, 1.0, 1.0).
         * 
         * The scale is defined as a set of three incremental changes along the three axes
         * (x,y,z). You can multiply each axis with a different number. When the scale
         * changes are applied to a display object, the object's size increases or decreases.
         * For example, setting the x, y, and z axes to two doubles the size of the object,
         * while setting the axes to 0.5 halves the size. To make sure that the scale
         * transformation only affects a specific axis, set the other parameters to one. A
         * parameter of one means no scale change along the specific axis.
         * 
         * The `appendScale()` method can be used for resizing as well as for managing
         * distortions, such as stretch or contract of a display object, or for zooming in
         * and out on a location. Scale transformations are automatically performed during a
         * display object's rotation and translation.
         * 
         * The order of transformation matters. A resizing followed by a translation
         * transformation produces a different effect than a translation followed by a
         * resizing transformation.
         * 
         * @param   xScale  A multiplier used to scale the object along the x axis.
         * @param   yScale  A multiplier used to scale the object along the y axis.
         * @param   zScale  A multiplier used to scale the object along the z axis.
         */
        // Based on https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L307
        public function appendScale(xScale:Number, yScale:Number, zScale:Number):void {
            this.append(new Matrix3D(Vector.<Number>([
                xScale, 0, 0, 0,
                0, yScale, 0, 0,
                0, 0, zScale, 0,
                0, 0, 0, 1
            ])));
        }

        /**
         * Prepends an incremental translation, a repositioning along the x, y, and z axes,
         * to a Matrix3D object. When the Matrix3D object is applied to a display object,
         * the matrix performs the translation changes before other transformations in the
         * Matrix3D object.
         * 
         * Translation specifies the distance the display object moves from its current
         * location along the x, y, and z axes. The `prependTranslation()` method sets the
         * translation as a set of three incremental changes along the three axes (x,y,z).
         * To have a translation change only a specific axis, set the other parameters to
         * zero. A zero parameter means no change along the specific axis.
         * 
         * The translation changes are not absolute. The effect is object-relative,
         * relative to the frame of reference of the original position and orientation.
         * To make an absolute change to the transformation matrix, use the `recompose()`
         * method. The order of transformation also matters. A translation followed by a
         * rotation transformation produces a different effect than a rotation followed by
         * a translation transformation. When prependTranslation() is used, the display
         * object continues to move in the direction it is facing, regardless of the other
         * transformations. For example, if a display object was facing toward a positive x
         * axis, it continues to move in the direction specified by the
         * `prependTranslation()` method, regardless of how the object has been rotated. To
         * make translation changes occur after other transformations, use the
         * `appendTranslation()` method.
         * 
         * @param   x   An incremental translation along the x axis.
         * @param   y   An incremental translation along the y axis.
         * @param   z   An incremental translation along the z axis.
         */
        public function prependTranslation(x:Number, y:Number, z:Number):void {
            var m = new Matrix3D();
            m.position = new Vector3D(x, y, z);
            this.prepend(m);
        }

        /**
         * Prepends an incremental rotation to a Matrix3D object. When the Matrix3D object is
         * applied to a display object, the matrix performs the rotation before other
         * transformations in the Matrix3D object.
         * 
         * The display object's rotation is defined by an axis, an incremental degree of
         * rotation around the axis, and an optional pivot point for the center of the
         * object's rotation. The axis can be any general direction. The common axes are the
         * `X_AXIS (Vector3D(1,0,0))`, `Y_AXIS (Vector3D(0,1,0))`, and
         * `Z_AXIS (Vector3D(0,0,1))`. In aviation terminology, the rotation about the y
         * axis is called yaw. The rotation about the x axis is called pitch. The rotation
         * about the z axis is called roll.
         * 
         * The order of transformation matters. A rotation followed by a translation
         * transformation produces a different effect than a translation followed by a
         * rotation.
         * 
         * The rotation effect is not absolute. The effect is object-relative, relative to
         * the frame of reference of the original position and orientation. To make an
         * absolute change to the transformation, use the `recompose()` method.
         * 
         * When the `prependRotation()` method's transformation is applied to a Matrix3D
         * object of a display object, the cached rotation property values of the display
         * object are invalidated.
         * 
         * One way to have a display object rotate around a specific point relative to its
         * location is to set the translation of the object to the specified point, rotate
         * the object using the `prependRotation()` method, and translate the object back to
         * the original position. In the following example, the `myObject` 3D display object
         * makes a y-axis rotation around the coordinate (10,10,0).
         * 
         * ```
         * myObject.z = 1;
         * myObject.transform.matrix3D.prependTranslation(10,10,0);
         * myObject.transform.matrix3D.prependRotation(1, Vector3D.Y_AXIS);
         * myObject.transform.matrix3D.prependTranslation(-10,-10,0);
         * ```
         * 
         * @param   degrees The degree of rotation.
         * @param   axis    The axis or direction of rotation. The usual axes are the
         * `X_AXIS (Vector3D(1,0,0))`, `Y_AXIS (Vector3D(0,1,0))`, and
         * `Z_AXIS (Vector3D(0,0,1))`. This vector should have a length of one.
         * @param   pivotPoint  A point that determines the center of rotation. The default
         * pivot point for an object is its registration point.
         */
        public function prependRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void {
            var m = new Matrix3D();
            m.appendRotation(degrees, axis, pivotPoint);
            this.prepend(m);
        }

        /**
         * A Vector3D object that holds the position, the 3D coordinate (x,y,z) of a display
         * object within the transformation's frame of reference. The `position` property
         * provides immediate access to the translation vector of the display object's
         * matrix without needing to decompose and recompose the matrix.
         * 
         * With the `position` property, you can get the translation elements of the
         * transformation matrix.
         */
        public function get position():Vector3D {
            return new Vector3D(this._rawData[12], this._rawData[13], this._rawData[14]);
        }

        /**
         * A Vector3D object that holds the position, the 3D coordinate (x,y,z) of a display
         * object within the transformation's frame of reference. The `position` property
         * provides immediate access to the translation vector of the display object's
         * matrix without needing to decompose and recompose the matrix.
         * 
         * With the `position` property, you set the translation elements of the
         * transformation matrix.
         */
        public function set position(val:Vector3D):void {
            this._rawData[12] = val.x;
            this._rawData[13] = val.y;
            this._rawData[14] = val.z;
        }

        /**
         * Prepends a matrix by multiplying the current Matrix3D object by another Matrix3D
         * object. The result combines both matrix transformations.
         * 
         * Matrix multiplication is different from matrix addition. Matrix multiplication is
         * not commutative. In other words, A times B is not equal to B times A. With the
         * `prepend()` method, the multiplication happens from the right side, meaning the `rhs`
         * Matrix3D object is on the right side of the multiplication operator.
         * 
         * ```
         * thisMatrix = thisMatrix * rhs
         * ```
         * 
         * The modifications made by `prepend()` method are object-space-relative. In other
         * words, they are always relative to the object's initial frame of reference.
         * 
         * The `prepend()` method replaces the current matrix with the prepended matrix. If
         * you want to prepend two matrixes without altering the current matrix, first copy
         * the current matrix by using the `clone()` method and then apply the `prepend()`
         * method to the copy.
         * 
         * @param   rhs A right-hand-side of the matrix by which the current Matrix3D is
         * multiplied.
         */
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

        /**
         * Prepends an incremental scale change along the x, y, and z axes to a Matrix3D
         * object. When the Matrix3D object is applied to a display object, the matrix
         * performs the scale changes before other transformations in the Matrix3D
         * object. The changes are object-relative, relative to the frame of reference of
         * the original position and orientation. The default scale factor is (1.0, 1.0, 1.0).
         * 
         * The scale is defined as a set of three incremental changes along the three
         * axes (x,y,z). You can multiply each axis with a different number. When the
         * scale changes are applied to a display object, the object's size increases or
         * decreases. For example, setting the x, y, and z axes to two doubles the size of
         * the object, while setting the axes to 0.5 halves the size. To make sure that the
         * scale transformation only affects a specific axis, set the other parameters to
         * one. A parameter of one means no scale change along the specific axis.
         * 
         * The `prependScale()` method can be used for resizing as well as for managing
         * distortions, such as stretch or contract of a display object. It can also be used
         * for zooming in and out on a location. Scale transformations are automatically
         * performed during a display object's rotation and translation.
         * 
         * The order of transformation matters. A resizing followed by a translation
         * transformation produces a different effect than a translation followed by a
         * resizing transformation.
         * 
         * @param   xScale  A multiplier used to scale the object along the x axis.
         * @param   yScale  A multiplier used to scale the object along the y axis.
         * @param   zScale  A multiplier used to scale the object along the z axis.
         */
        public function prependScale(xScale:Number, yScale:Number, zScale:Number):void {
            var m = new Matrix3D();
            m.appendScale(xScale, yScale, zScale);
            this.prepend(m);
        }

        /**
         * Copies all of the matrix data from the source Matrix3D object into the calling
         * Matrix3D object.
         * 
         * @param   sourceMatrix3D  The Matrix3D object from which to copy the data.
         */
        [API("674")]
        public function copyFrom(other:Matrix3D):void {
            // This makes a copy of other.rawData
            this._rawData = other.rawData;
        }

        /**
         * Copies all of the matrix data from the calling Matrix3D object into the
         * provided vector. The optional index parameter allows you to select any target
         * starting slot in the vector.
         * 
         * @param   vector  The vector object to which to copy the data.
         * @param   index
         * @param   transpose
         */
        [API("674")]
        public function copyRawDataTo(vector:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void {
            if (transpose) {
                this.transpose();
            }

            for (var i = 0; i < rawData.length; i++) {
                vector[i + index] = this._rawData[i];
            }

            if (transpose) {
                this.transpose();
            }
        }

        /**
         * Returns a new Matrix3D object that is an exact copy of the current Matrix3D object.
         * 
         * @return  A new Matrix3D object that is an exact copy of the current Matrix3D
         * object.
         */
        [Ruffle(NativeCallable)]
        public function clone():Matrix3D {
            return new Matrix3D(this.rawData);
        }

        /**
         * @param   other
         */
        public function copyToMatrix3D(other:Matrix3D):void {
            other.rawData = this.rawData;
        }

        /**
         * Rotates the display object so that it faces a specified position. This method
         * allows for an in-place modification to the orientation. The forward direction
         * vector of the display object (the at Vector3D object) points at the specified
         * world-relative position. The display object's up direction is specified with the
         * up Vector3D object.
         * 
         * The `pointAt()` method invalidates the cached rotation property value of the
         * display object. The method decomposes the display object's matrix and modifies the
         * rotation elements to have the object turn to the specified position. It then
         * recomposes (updates) the display object's matrix, which performs the
         * transformation. If the object is pointing at a moving target, such as a moving
         * object's position, then with each subsequent call, the method has the object
         * rotate toward the moving target.
         * 
         * **Note:** If you use the `Matrix3D.pointAt()` method without setting the
         * optional parameters, a target object does not face the specified world-relative
         * position by default. You need to set the values for at to the -y-axis (0,-1,0)
         * and up to the -z axis (0,0,-1). ** and where does it point to ???
         * 
         * @param   pos The world-relative position of the target object. World-relative
         * defines the object's transformation relative to the world space and coordinates,
         * where all objects are positioned.
         * @param   at  The object-relative vector that defines where the display object is
         * pointing. Object-relative defines the object's transformation relative to the
         * object space, the object's own frame of reference and coordinate system. Default
         * value is the +y axis (0,1,0).
         * @param   up  The object-relative vector that defines "up" for the display object.
         * If the object is drawn looking down from above, the +z axis is its "up" vector.
         * Object-relative defines the object's transformation relative to the object space,
         * the object's own frame of reference and coordinate system. Default value is the
         * +z-axis (0,0,1).
         */
        public function pointAt(pos:Vector3D, at:Vector3D = null, up:Vector3D = null):void {
            if (at == null) {
                at = new Vector3D(0, 0, 1); // Default aiming direction (forward along the Z axis)
            }

            if (up == null) {
                up = new Vector3D(0, 1, 0);  // Default Up direction (up along the Y axis)
            }

            // if the method is called with pos only, so that at and up is null
            // flash does something i don't understand...
            // it should use these vectors here => Default At and Default Up
            // no idea what values are in use... when setting at and up everything looks really fine...

            // if up and at makes no sense flash return a identity matrix
            // if hope it does this only for 0,1,2  4,5,6  8,9,10

            //trace('args', pos, at, up);

            // Decompose the current matrix into components
            var components:Vector.<Vector3D> = this.decompose();  // 0 = translation, 1 = rotation, 2 = scale
            var translation:Vector3D = components[0];
            var scale:Vector3D = components[2];

            // we want our eye
            var eye:Vector3D = pos.subtract(translation); // don't normalize eye afterwards...
            if (at.z <= 0) {
                eye.negate();
            }
            //trace('eye', eye)

            // flash does here something that is not clear... if at has x,y other then 0
            // there is an additional transformation on the vector * logic unknown...

            // we should work here with normalized at and up, leave the args untouched...
            var atNorm:Vector3D = at.clone();
            var upNorm:Vector3D = up.clone();

            // .w is irrelavant when calling normalize ? looks so... Vector3D.normalize() ignores .w
            atNorm.normalize();
            upNorm.normalize();

            // Calculate the Z axis (the direction to the target)
            var zAxis:Vector3D = atNorm.subtract(eye); 
            zAxis.normalize();

            // the axis must be orthogonal

            // Clone the Up vector
            var upVector:Vector3D = upNorm.clone();

            if (1) { // not sure if needed...
                // Adjust the Up vector to ensure it's orthogonal to the Z axis
                var dirProjection:Vector3D = zAxis.clone();
                dirProjection.scaleBy(upVector.dotProduct(zAxis));  // Project upVector onto the zAxis

                upVector = upVector.subtract(dirProjection);  // Subtract projection from upVector to make it orthogonal

                // Check if the Up vector is valid, otherwise create a default orthogonal vector
                if (upVector.length > 0) {
                    upVector.normalize();  // Normalize if it's valid
                } else {
                    // Create an orthogonal vector if Up and direction are too similar
                    if (zAxis.x != 0) {
                        upVector = new Vector3D(-zAxis.y, zAxis.x, 0);  // Example orthogonal vector in XY plane
                    } else {
                        upVector = new Vector3D(1, 0, 0);  // Fallback orthogonal vector
                    }
                    //trace('upVector');
                }
            }

            // Calculate the X-axis (right vector, by cross product of Up and Z-axis)
            var xAxis:Vector3D = upVector.crossProduct(zAxis);
            xAxis.normalize();

            // Calculate the Y-axis (Up vector, by cross product of Z-axis and X-axis)
            var yAxis:Vector3D = zAxis.crossProduct(xAxis);
            yAxis.normalize();
            yAxis.negate(); // bring it to flash 3d space

            // we should use recompose here... but then we need a quaternion
            // see
            // https://www.w3.org/TR/css-transforms-2/#matrix-interpolation

            var mr:Vector.<Number> = this.rawData;

            // we leave perspective untouched... [3,7,11,15]
            // and translation [12,13,14] is allready set... no need to set it again

            mr[0] = xAxis.x;
            mr[1] = xAxis.y;
            mr[2] = xAxis.z;
            //mr[3] = 0; 

            mr[4] = yAxis.x;
            mr[5] = yAxis.y;
            mr[6] = yAxis.z;
            //mr[7] = 0;

            mr[8] = zAxis.x;
            mr[9] = zAxis.y;
            mr[10] = zAxis.z;
            //mr[11] = 0;

            //mr[15] = 1;

            // Apply skew (if provided)
            if (components.length > 3) {
                var skew:Vector3D = components[3];
                //trace('skew', skew);

                // Apply XY skew
                mr[4] += mr[0] * skew.x;
                mr[5] += mr[1] * skew.x;
                mr[6] += mr[2] * skew.x;

                // Apply XZ skew
                mr[8] += mr[0] * skew.y;
                mr[9] += mr[1] * skew.y;
                mr[10] += mr[2] * skew.y;

                // Apply YZ skew
                mr[8] += mr[4] * skew.z;
                mr[9] += mr[5] * skew.z;
                mr[10] += mr[6] * skew.z;
            }

            // Apply scale to X, Y, Z axes
            for (var i:int = 0; i < 3; i++) {
                mr[i] *= scale.x;  // X-axis
                mr[4 + i] *= scale.y;  // Y-axis
                mr[8 + i] *= scale.z;  // Z-axis
            }

            this.rawData = mr;
        }

        /**
         * Interpolates this matrix towards the translation, rotation, and scale
         * transformations of the target matrix.
         * 
         * The `interpolateTo()` method avoids the unwanted results that can occur when
         * using methods such as the display object's axis rotation properties. The
         * `interpolateTo()` method invalidates the cached value of the rotation property of
         * the display object and converts the orientation elements of the display object's
         * matrix to a quaternion before interpolation. This method guarantees the shortest,
         * most efficient path for the rotation. It also produces a smooth, gimbal-lock-free
         * rotation. A gimbal lock can occur when using Euler Angles, where each axis is
         * handled independently. During the rotation around two or more axes, the axes can
         * become aligned, leading to unexpected results. Quaternion rotation avoids the
         * gimbal lock.
         * 
         * **Note:** In case of interpolation, the scaling value of the matrix will reset and
         * the matrix will be normalized. (which is wrong)
         * 
         * Consecutive calls to the `interpolateTo()` method can produce the effect of a
         * display object starting quickly and then slowly approaching another display
         * object. For example, if the percent parameter is set to 0.1, the display object
         * moves ten percent toward the target object specified by the `toMat` parameter.
         * On subsequent calls or in subsequent frames, the object moves ten percent of the
         * remaining 90 percent, then ten percent of the remaining distance, and continues
         * until it reaches the target.
         * 
         * @param   toMat   The target Matrix3D object.
         * @param   percent A value between 0 and 1 that determines the location of the
         * display object relative to the target. The closer the value is to 1.0, the closer
         * the display object is to its current position. The closer the value is to 0, the
         * closer the display object is to the target.
         */
        public function interpolateTo(toMat:Matrix3D, percent:Number):void {
            // beware this logic is different to method interpolate, 0 means we are at the toMat
            // so we start here with the toMat...
            var m:Matrix3D = Matrix3D.interpolate(toMat, new Matrix3D(this.rawData), percent);
            this.rawData = m.rawData;
        }

        /**
         * Interpolates the translation, rotation, and scale transformation of one matrix
         * toward those of the target matrix.
         * 
         * The `interpolate()` method avoids some of the unwanted results that can occur
         * when using methods such as the display object's axis rotation properties. The
         * `interpolate()` method invalidates the cached value of the rotation property of
         * the display object and converts the orientation elements of the display object's
         * matrix to a quaternion before interpolation. This method guarantees the shortest,
         * most efficient path for the rotation. It also produces a smooth, gimbal-lock-free
         * rotation. A gimbal lock can occur when using Euler Angles, where each axis is
         * handled independently. During the rotation around two or more axes, the axes can
         * become aligned, leading to unexpected results. Quaternion rotation avoids the
         * gimbal lock.
         * 
         * Consecutive calls to the `interpolate()` method can produce the effect of a
         * display object starting quickly and then slowly approaching another display
         * object. For example, if you set the `thisMat` parameter to the returned Matrix3D
         * object, the `toMat` parameter to the target display object's associated Matrix3D
         * object, and the `percent` parameter to 0.1, the display object moves ten percent
         * toward the target object. On subsequent calls or in subsequent frames, the object
         * moves ten percent of the remaining 90 percent, then ten percent of the remaining
         * distance, and continues until it reaches the target.
         * 
         * @param   thisMat The Matrix3D object that is to be interpolated.
         * @param   toMat   The target Matrix3D object.
         * @param   percent A value between 0 and 1 that determines the percent the
         * `thisMat` Matrix3D object is interpolated toward the target Matrix3D object.
         * @return  A Matrix3D object with elements that place the values of the matrix
         * between the original matrix and the target matrix. When the returned matrix is
         * applied to the this display object, the object moves the specified percent closer
         * to the target object.
         */
        public static function interpolate(thisMat:Matrix3D, toMat:Matrix3D, percent:Number):Matrix3D {

            var debug:Boolean = false;

            // implemented a 'static function correct' with default value false
            // to keep it compatible to older projects relying on the former logic...
            
            var method:String = Orientation3D.QUATERNION;

            var decomposedA:Vector.<Vector3D>, decomposedB:Vector.<Vector3D>;

            // i pretty sure, this will be slow, decompose might take too much time...
            // simple speed up, create a small cache for the last 10 matrix mr which stores the 
            // decompose data for thisMat and toMat
            // useless when interpolating many different mr

            decomposedA = thisMat.decompose(method);
            decomposedB = toMat.decompose(method);

            // 0 = translation, 1 = rotation, 2 = scale
            // 3 = skew, 4 = perspective * own implementation
            //trace('skew', decomposedA[3], decomposedB[3]);
            //trace('pers', decomposedA[4], decomposedB[4]);

            if (debug) trace('scales', decomposedA[2], decomposedB[2]);
            
            var v0:Vector3D = decomposedA[1];
            var v1:Vector3D = decomposedB[1];
            
            var dot:Number = v0.x * v1.x + v0.y * v1.y + v0.z * v1.z + v0.w * v1.w; // we reuse this later below...
            var cosOmega:Number = dot;

            if (cosOmega < 0) {
                cosOmega = -cosOmega;
            }

            var k0:Number, k1:Number;

            if (cosOmega > 0.9999 || percent == 0 || percent == 1) {
                // If the quaternions are nearly identical, perform a linear interpolation
                k0 = 1 - percent;
                k1 = percent;
                if (debug) trace('linear');
            } else {
                // Otherwise, use spherical linear interpolation (Slerp)
                var sinOmega:Number = Math.sqrt(1 - cosOmega * cosOmega);
                var omega:Number = Math.atan2(sinOmega, cosOmega);
                var oneOverSinOmega:Number = 1 / sinOmega;
                k0 = Math.sin((1 - percent) * omega) * oneOverSinOmega;
                k1 = Math.sin(percent * omega) * oneOverSinOmega;
                if (debug) trace('slerp');
            }
            if (debug) trace('k0', k0, 'k1', k1);

            // beware flash does not use slerp everywhere...
            // so far i've checked the logic it's only used for the quaternion
            
            var tx:Number, ty:Number, tz:Number;
            // pretty sure this is what flash does... lerp... yes...
            tx = decomposedA[0].x * (1 - percent) + decomposedB[0].x * percent;
            ty = decomposedA[0].y * (1 - percent) + decomposedB[0].y * percent;
            tz = decomposedA[0].z * (1 - percent) + decomposedB[0].z * percent;

            var x:Number, y:Number, z:Number, w:Number;
            
            if (debug) trace('v0', v0, v0.w); // for the first vector * rotation
            if (debug) trace('v1', v1, v1.w); // for the second vector * rotation

            // If the angle is greater than 180 degrees, negate the quaternion to take the shorter path
            if (dot < 0.0) { 
                // all parts of the quaternion must be touched... don't use .negate(); .w is not touched...
                if (debug) trace('quaternion v1 inverted');
                v1.x = -v1.x;
                v1.y = -v1.y;
                v1.z = -v1.z;
                v1.w = -v1.w;
            }

            x = v0.x * k0 + v1.x * k1;
            y = v0.y * k0 + v1.y * k1;
            z = v0.z * k0 + v1.z * k1;
            w = v0.w * k0 + v1.w * k1;

            // Normalization of the quaternion, otherwise we could get a not wanted scale effects
            if (1) {
                var magnitude:Number = Math.sqrt(x * x + y * y + z * z + w * w);
                x /= magnitude;
                y /= magnitude;
                z /= magnitude;
                w /= magnitude;
            }

            var scale = new Vector3D();
            // lerp... yes...
            scale.x = decomposedA[2].x * (1 - percent) + decomposedB[2].x * percent;
            scale.y = decomposedA[2].y * (1 - percent) + decomposedB[2].y * percent;
            scale.z = decomposedA[2].z * (1 - percent) + decomposedB[2].z * percent;

            if (debug) trace('scale', scale); // not in use by flash...

            var skew:Vector3D = new Vector3D(0, 0, 0);
            if (decomposedA.length > 3 && decomposedB.length > 3) { // if provided
                // lerp
                skew.x = decomposedA[3].x * (1 - percent) + decomposedB[3].x * percent;
                skew.y = decomposedA[3].y * (1 - percent) + decomposedB[3].y * percent;
                skew.z = decomposedA[3].z * (1 - percent) + decomposedB[3].z * percent;
            }
            if (debug) trace('skew', skew); // not in use by flash...

            var perspective:Vector3D = new Vector3D(0, 0, 0, 1);
            if (decomposedA.length > 4 && decomposedB.length > 4) { // if provided
                // lerp
                perspective.x = decomposedA[4].x * (1 - percent) + decomposedB[4].x * percent;
                perspective.y = decomposedA[4].y * (1 - percent) + decomposedB[4].y * percent;
                perspective.z = decomposedA[4].z * (1 - percent) + decomposedB[4].z * percent;
                perspective.w = decomposedA[4].w * (1 - percent) + decomposedB[4].w * percent;

                // should we normalize ???
            }
            if (debug) trace('perspective', perspective, perspective.w); // not in use by flash...


            // let recompose do our work
            var m:Matrix3D = new Matrix3D();
            var vec = new Vector.<Vector3D>([]);

            vec.push(new Vector3D(tx, ty, tz)); // translation
            vec.push(new Vector3D(x, y, z, w)); // rotation
            if (Matrix3D.correct) {
                vec.push(scale); // scale
                vec.push(skew); // skew
                vec.push(perspective); // perspective
            } else {
                vec.push(new Vector3D(1, 1, 1)); // scale
            }

            m.recompose(vec, method);

            if (debug) trace('mr', m.rawData);

            return m; 

/*          
            // double code not needed anymore...

            var mr:Vector.<Number> = new Vector.<Number>(16, true);
            
            // recompose matrix raw with quaternion (without 12,13,14 this is basically quaternion to matrix)
            mr[0] = (1 - 2 * y * y - 2 * z * z);
            mr[1] = (2 * x * y + 2 * w * z);
            mr[2] = (2 * x * z - 2 * w * y);
            mr[3] = 0;
            mr[4] = (2 * x * y - 2 * w * z);
            mr[5] = (1 - 2 * x * x - 2 * z * z);
            mr[6] = (2 * y * z + 2 * w * x);
            mr[7] = 0;
            mr[8] = (2 * x * z + 2 * w * y);
            mr[9] = (2 * y * z - 2 * w * x);
            mr[10] = (1 - 2 * x * x - 2 * y * y);
            mr[11] = 0;
            mr[12] = tx;
            mr[13] = ty;
            mr[14] = tz;
            mr[15] = 1;

            // note: handle scale, skew and perspective
            // skew and perspective is available in extended decompose now
            // just do a simple lerp for skew and perspective * done... but not tested...

            if (Matrix3D.correct) {

                // Apply skew * tbd testing

                // Apply XY skew
                mr[4] += mr[0] * skew.x;
                mr[5] += mr[1] * skew.x;
                mr[6] += mr[2] * skew.x;

                // Apply XZ skew
                mr[8] += mr[0] * skew.y;
                mr[9] += mr[1] * skew.y;
                mr[10] += mr[2] * skew.y;

                // Apply YZ skew
                mr[8] += mr[4] * skew.z;
                mr[9] += mr[5] * skew.z;
                mr[10] += mr[6] * skew.z;

                if (debug) trace('skew mr', mr);

                // Apply scale
                for (var i:int = 0; i < 3; i++) {
                    mr[i] *= scale.x;  // X-axis
                    mr[4 + i] *= scale.y;  // Y-axis
                    mr[8 + i] *= scale.z;  // Z-axis
                }

                if (debug) trace('scaled mr', mr);

                // Apply perspective * tbd testing

                mr[3] = perspective.x;
                mr[7] = perspective.y;
                mr[11] = perspective.z;
                mr[15] = perspective.w;

                if (debug) trace('perspective mr', mr);


            }

            // ??? should we avoid 0 scales like in recompose ???

            var m:Matrix3D = new Matrix3D(mr);
            
            return m;
*/
        }

        /**
         * Sets the transformation matrix's translation, rotation, and scale settings. Unlike
         * the incremental changes made by the display object's rotation properties or
         * Matrix3D object's rotation methods, the changes made by `recompose()` method are
         * absolute changes. The `recompose()` method overwrites the matrix's transformation.
         * 
         * To modify the matrix's transformation with an absolute parent frame of reference,
         * retrieve the settings with the decompose() method and make the appropriate
         * changes. You can then set the Matrix3D object to the modified transformation
         * using the `recompose()` method.
         * 
         * The `recompose()` method's parameter specifies the orientation style that was
         * used for the transformation. The default orientation is eulerAngles, which defines
         * the orientation with three separate angles of rotation for each axis. The
         * rotations occur consecutively and do not change the axis of each other. The
         * display object's axis rotation properties perform Euler Angles orientation style
         * transformation. The other orientation style options are axisAngle and quaternion.
         * The Axis Angle orientation uses the combination of an axis and an angle to
         * determine the orientation. The axis around which the object is rotated is a unit
         * vector that represents a direction. The angle represents the magnitude of the
         * rotation about the vector. The direction also determines where a display object
         * is facing and the angle determines which way is up. The `appendRotation()` and
         * `prependRotation()` methods use the Axis Angle orientation. The quaternion
         * orientation uses complex numbers and the fourth element of a vector. An
         * orientation is represented by the three axes of rotation (x,y,z) and an angle of
         * rotation (w). The interpolate() method uses quaternion.
         * 
         * @param   components  A Vector of three Vector3D objects that replace the Matrix3D
         * object's translation[0], rotation[1], scale[2], and optional skew[3] and perspective[4] elements
         * @param   orientationStyle    An optional parameter that determines the orientation
         * style used for the matrix transformation. The three types of orientation styles
         * are eulerAngles (constant `EULER_ANGLES`), axisAngle (constant `AXIS_ANGLE`), and
         * quaternion (constant `QUATERNION`). For additional information on the different
         * orientation style, see the geom.Orientation3D class.
         * @return  Returns `false` if any of the Vector3D elements of the components
         * Vector do not exist or are `null`.
         */
        // Based on OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L1437
        public function recompose(components:Vector.<Vector3D>, orientationStyle:String = "eulerAngles"):Boolean {

            if (!(orientationStyle == Orientation3D.AXIS_ANGLE || orientationStyle == Orientation3D.EULER_ANGLES || orientationStyle == Orientation3D.QUATERNION)) {
                throw new Error("Error #2187: Invalid orientation style " + orientationStyle + ".  Value must be one of 'Orientation3D.EULER_ANGLES', 'Orientation3D.AXIS_ANGLE', or 'Orientation3D.QUATERNION'.", 2187);
            }

            var rot:Vector3D = components[1];

            if (orientationStyle == Orientation3D.QUATERNION) {
                // Flash throws exceptions from 'recompose' certain values of 'components'
                // which we need to reproduce. See the 'matrix3d_compose' test

                // Check if .w is defined * not sure if this ever happens... just to be sure...
                if (isNaN(rot.w)) {
                    throw new Error("Error #2187: Invalid orientation style " + orientationStyle + ". Invalid quaternion: 'w' component is undefined.");
                }

                // Normalize the quaternion (if it's not already normalized)
                var length:Number = Math.sqrt(rot.x * rot.x + rot.y * rot.y + rot.z * rot.z + rot.w * rot.w);

                // If the length is zero or not valid, the quaternion is invalid
                if (length == 0) {
                    throw new Error("Error #2187: Invalid orientation style " + orientationStyle + ". Invalid quaternion: zero length.");
                }

                // If the quaternion is not normalized, normalize it
                if (length != 1) {
                    rot.x /= length;
                    rot.y /= length;
                    rot.z /= length;
                    rot.w /= length;
                }
            }

            // RUFFLE - unlike in OpenFL, we continue on even if some of the 'scale' components are 0
            if (components.length < 3) {
                return false;
            }

            this.identity(); // if anything goes wrong, should we stay with the identity matrix ??

            // 0 = translation, 1 = rotation, 2 = scale
            var tx:Number = components[0].x;
            var ty:Number = components[0].y;
            var tz:Number = components[0].z;

            // rotation see above...

            var scale = new Vector3D();
            scale.x = components[2].x;
            scale.y = components[2].y;
            scale.z = components[2].z;

            var mr:Vector.<Number> = this.rawData;

            switch (orientationStyle) {
                case Orientation3D.EULER_ANGLES:
                    var cx:Number = Math.cos(rot.x);
                    var cy:Number = Math.cos(rot.y);
                    var cz:Number = Math.cos(rot.z);
                    var sx:Number = Math.sin(rot.x);
                    var sy:Number = Math.sin(rot.y);
                    var sz:Number = Math.sin(rot.z);

                    mr[0] = cy * cz;
                    mr[1] = cy * sz;
                    mr[2] = -sy;
                    mr[3] = 0;
                    mr[4] = (sx * sy * cz - cx * sz);
                    mr[5] = (sx * sy * sz + cx * cz);
                    mr[6] = sx * cy;
                    mr[7] = 0;
                    mr[8] = (cx * sy * cz + sx * sz);
                    mr[9] = (cx * sy * sz - sx * cz);
                    mr[10] = cx * cy;
                    mr[11] = 0;
                    mr[12] = tx;
                    mr[13] = ty;
                    mr[14] = tz;
                    mr[15] = 1;
                    break;
                case Orientation3D.QUATERNION:
                default:
                    var x:Number = rot.x;
                    var y:Number = rot.y;
                    var z:Number = rot.z;
                    var w:Number = rot.w;

                    if (orientationStyle == Orientation3D.AXIS_ANGLE) {
                        x *= Math.sin(w / 2);
                        y *= Math.sin(w / 2);
                        z *= Math.sin(w / 2);
                        w = Math.cos(w / 2);
                    }

                    mr[0] = (1 - 2 * y * y - 2 * z * z);
                    mr[1] = (2 * x * y + 2 * w * z);
                    mr[2] = (2 * x * z - 2 * w * y);
                    mr[3] = 0;
                    mr[4] = (2 * x * y - 2 * w * z);
                    mr[5] = (1 - 2 * x * x - 2 * z * z);
                    mr[6] = (2 * y * z + 2 * w * x);
                    mr[7] = 0;
                    mr[8] = (2 * x * z + 2 * w * y);
                    mr[9] = (2 * y * z - 2 * w * x);
                    mr[10] = (1 - 2 * x * x - 2 * y * y);
                    mr[11] = 0;
                    mr[12] = tx;
                    mr[13] = ty;
                    mr[14] = tz;
                    mr[15] = 1;
            }

            // note the order is extremly important, skew must be applied before scale

            // Apply skew (if provided)
            if (components.length > 3) {
                var skew:Vector3D = components[3];

                // Apply XY skew
                mr[4] += mr[0] * skew.x;
                mr[5] += mr[1] * skew.x;
                mr[6] += mr[2] * skew.x;

                // Apply XZ skew
                mr[8] += mr[0] * skew.y;
                mr[9] += mr[1] * skew.y;
                mr[10] += mr[2] * skew.y;

                // Apply YZ skew
                mr[8] += mr[4] * skew.z;
                mr[9] += mr[5] * skew.z;
                mr[10] += mr[6] * skew.z;
            }

            // Apply scale
            for (var i:int = 0; i < 3; i++) {
                mr[i] *= scale.x;  // X-axis
                mr[4 + i] *= scale.y;  // Y-axis
                mr[8 + i] *= scale.z;  // Z-axis
            }

            // Apply perspective (if provided)
            if (components.length > 4) {
                var perspective:Vector3D = components[4];

                mr[3] = perspective.x;
                mr[7] = perspective.y;
                mr[11] = perspective.z;
                mr[15] = perspective.w;
            }

            // Avoid 0 scales
            if (scale.x == 0) mr[0] = 1e-15;
            if (scale.y == 0) mr[5] = 1e-15;
            if (scale.z == 0) mr[10] = 1e-15;

            this.rawData = mr;

            return !(scale.x == 0 || scale.y == 0 || scale.y == 0);
        }

        /**
         * Copies specific column of the calling Matrix3D object into the Vector3D object.

         * @param   column  The column from which to copy the data.
         * @param   vector3D    The destination Vector3D object of the copy.
         */
        [API("674")]
        public function copyColumnTo(column:uint, vector3D:Vector3D):void {
            if (column > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            switch (column) {
                case 0:
                    vector3D.x = this._rawData[0];
                    vector3D.y = this._rawData[1];
                    vector3D.z = this._rawData[2];
                    vector3D.w = this._rawData[3];
                    break;

                case 1:
                    vector3D.x = this._rawData[4];
                    vector3D.y = this._rawData[5];
                    vector3D.z = this._rawData[6];
                    vector3D.w = this._rawData[7];
                    break;

                case 2:
                    vector3D.x = this._rawData[8];
                    vector3D.y = this._rawData[9];
                    vector3D.z = this._rawData[10];
                    vector3D.w = this._rawData[11];
                    break;

                case 3:
                    vector3D.x = this._rawData[12];
                    vector3D.y = this._rawData[13];
                    vector3D.z = this._rawData[14];
                    vector3D.w = this._rawData[15];
                    break;
            }
        }

        /**
         * Copies a Vector3D object into specific column of the calling Matrix3D object.
         * 
         * @param   column  The destination column of the copy.
         * @param   vector3D    The Vector3D object from which to copy the data.
         */
        [API("674")]
        public function copyColumnFrom(column:uint, vector3D:Vector3D):void {
            if (column > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            switch (column) {
                case 0:
                    this._rawData[0] = vector3D.x;
                    this._rawData[1] = vector3D.y;
                    this._rawData[2] = vector3D.z;
                    this._rawData[3] = vector3D.w;
                    break;

                case 1:
                    this._rawData[4] = vector3D.x;
                    this._rawData[5] = vector3D.y;
                    this._rawData[6] = vector3D.z;
                    this._rawData[7] = vector3D.w;
                    break;

                case 2:
                    this._rawData[8] = vector3D.x;
                    this._rawData[9] = vector3D.y;
                    this._rawData[10] = vector3D.z;
                    this._rawData[11] = vector3D.w;
                    break;

                case 3:
                    this._rawData[12] = vector3D.x;
                    this._rawData[13] = vector3D.y;
                    this._rawData[14] = vector3D.z;
                    this._rawData[15] = vector3D.w;
                    break;
            }
        }

        /**
         * Returns the transformation matrix's translation, rotation, and scale settings as
         * a Vector of three Vector3D objects. The first Vector3D object holds the
         * translation elements. The second Vector3D object holds the rotation elements.
         * The third Vector3D object holds the scale elements.
         * 
         * Some Matrix3D methods, such as the `interpolateTo()` method, automatically
         * decompose and recompose the matrix to perform their transformation.
         * 
         * To modify the matrix's transformation with an absolute parent frame of reference,
         * retrieve the settings with the `decompose()` method and make the appropriate
         * changes. You can then set the Matrix3D object to the modified transformation
         * using the `recompose()` method.
         * 
         * The `decompose()` method's parameter specifies the orientation style that is
         * meant to be used for the transformation. The default orientation is `eulerAngles`,
         * which defines the orientation with three separate angles of rotation for each
         * axis. The rotations occur consecutively and do not change the axis of each other.
         * The display object's axis rotation properties perform Euler Angles orientation
         * style transformation. The other orientation style options are `axisAngle` and
         * `quaternion`. The Axis Angle orientation uses a combination of an axis and an
         * angle to determine the orientation. The axis around which the object is rotated
         * is a unit vector that represents a direction. The angle represents the magnitude
         * of the rotation about the vector. The direction also determines where a display
         * object is facing and the angle determines which way is up. The `appendRotation()`
         * and `prependRotation()` methods use the Axis Angle orientation. The `quaternion`
         * orientation uses complex numbers and the fourth element of a vector. The three
         * axes of rotation (x,y,z) and an angle of rotation (w) represent the orientation.
         * The `interpolate()` method uses quaternion.
         * 
         * @param   orientationStyle    An optional parameter that determines the orientation
         * style used for the matrix transformation. The three types of orientation style
         * are `eulerAngles` (constant `EULER_ANGLES`), `axisAngle` (constant `AXIS_ANGLE`),
         * and `quaternion` (constant `QUATERNION`). For additional information on the
         * different orientation style, see the geom.Orientation3D class.
         * @return  A Vector of three Vector3D objects, each holding the translation,
         * rotation, and scale settings, respectively.
         */
        public function decompose(orientationStyle:String = "eulerAngles"):Vector.<Vector3D> {

            if (!(orientationStyle == Orientation3D.AXIS_ANGLE || orientationStyle == Orientation3D.EULER_ANGLES || orientationStyle == Orientation3D.QUATERNION)) {
                throw new Error("Error #2187: Invalid orientation style " + orientationStyle + ".  Value must be one of 'Orientation3D.EULER_ANGLES', 'Orientation3D.AXIS_ANGLE', or 'Orientation3D.QUATERNION'.", 2187);
            }

            var mr = this.rawData;

            var pos = new Vector3D(mr[12], mr[13], mr[14]);
            mr[12] = 0;
            mr[13] = 0;
            mr[14] = 0;

            var perspective = new Vector3D(0, 0, 0, 1);

            if (mr[3] != 0 || mr[7] != 0 || mr[11] != 0) {

                if (1) {

                    perspective.x = mr[3];
                    perspective.y = mr[7];
                    perspective.z = mr[11];
                    perspective.w = mr[15];

                    mr[3] = 0;
                    mr[7] = 0;
                    mr[11] = 0;
                    mr[15] = 1;

                } else {

                    var rhs = new Vector3D(mr[3], mr[7], mr[11], mr[15]);

                    // Remove the perspective values from the matrix
                    mr[3] = 0;
                    mr[7] = 0;
                    mr[11] = 0;
                    mr[15] = 1; // Set the Homogeneous value to 1

                    // Invert the matrix
                    var inversePerspectiveMatrix = this.clone(); // clone needed here...
                    inversePerspectiveMatrix.invert();

                    // Transpose the inverted matrix
                    var transposedInverse = inversePerspectiveMatrix.transpose();

                    // Calculate the perspective by multiplying by the transposed matrix
                    perspective = transposedInverse.transformVector(rhs);

                }
            }

            // note the order is extremly important, scale must be removed before skew 

            var scale = new Vector3D();

            scale.x = Math.sqrt(mr[0] * mr[0] + mr[1] * mr[1] + mr[2] * mr[2]);
            scale.y = Math.sqrt(mr[4] * mr[4] + mr[5] * mr[5] + mr[6] * mr[6]);
            scale.z = Math.sqrt(mr[8] * mr[8] + mr[9] * mr[9] + mr[10] * mr[10]);

            if (mr[0] * (mr[5] * mr[10] - mr[6] * mr[9]) - mr[1] * (mr[4] * mr[10] - mr[6] * mr[8]) + mr[2] * (mr[4] * mr[9] - mr[5] * mr[8]) < 0) {
                scale.z = -scale.z;
            }

            mr[0] /= scale.x;
            mr[1] /= scale.x;
            mr[2] /= scale.x;
            mr[4] /= scale.y;
            mr[5] /= scale.y;
            mr[6] /= scale.y;
            mr[8] /= scale.z;
            mr[9] /= scale.z;
            mr[10] /= scale.z;

            var skew = new Vector3D(0, 0, 0);
            // Calculate skew.x (XY scissor factor)
            skew.x = mr[0] * mr[4] + mr[1] * mr[5] + mr[2] * mr[6];
            mr[4] -= mr[0] * skew.x; // Remove the XY scissor factor from the Y axis
            mr[5] -= mr[1] * skew.x;
            mr[6] -= mr[2] * skew.x;

            //  Calculate skew.y (XZ scissor factor)
            skew.y = mr[0] * mr[8] + mr[1] * mr[9] + mr[2] * mr[10];
            mr[8] -= mr[0] * skew.y; // Remove the XZ scissor factor from the Z axis
            mr[9] -= mr[1] * skew.y;
            mr[10] -= mr[2] * skew.y;

            // Calculate skew.z (YZ scissor factor)
            skew.z = mr[4] * mr[8] + mr[5] * mr[9] + mr[6] * mr[10];
            mr[8] -= mr[4] * skew.z; // Remove the YZ scissor factor from the Z axis
            mr[9] -= mr[5] * skew.z;
            mr[10] -= mr[6] * skew.z;

            var rot = new Vector3D();

            switch (orientationStyle) {
                case Orientation3D.AXIS_ANGLE:
                    rot.w = Math.acos((mr[0] + mr[5] + mr[10] - 1) / 2);

                    var len = Math.sqrt((mr[6] - mr[9]) * (mr[6] - mr[9]) + (mr[8] - mr[2]) * (mr[8] - mr[2]) + (mr[1] - mr[4]) * (mr[1] - mr[4]));

                    if (len != 0) {
                        rot.x = (mr[6] - mr[9]) / len;
                        rot.y = (mr[8] - mr[2]) / len;
                        rot.z = (mr[1] - mr[4]) / len;
                    }
                    else {
                        rot.x = rot.y = rot.z = 0;
                    }
                    break;

                case Orientation3D.QUATERNION:
                    var tr = mr[0] + mr[5] + mr[10];

                    if (tr > 0) {
                        rot.w = Math.sqrt(1 + tr) / 2;

                        rot.x = (mr[6] - mr[9]) / (4 * rot.w);
                        rot.y = (mr[8] - mr[2]) / (4 * rot.w);
                        rot.z = (mr[1] - mr[4]) / (4 * rot.w);
                    }
                    else if ((mr[0] > mr[5]) && (mr[0] > mr[10])) {
                        rot.x = Math.sqrt(1 + mr[0] - mr[5] - mr[10]) / 2;

                        rot.w = (mr[6] - mr[9]) / (4 * rot.x);
                        rot.y = (mr[1] + mr[4]) / (4 * rot.x);
                        rot.z = (mr[8] + mr[2]) / (4 * rot.x);
                    }
                    else if (mr[5] > mr[10]) {
                        rot.y = Math.sqrt(1 + mr[5] - mr[0] - mr[10]) / 2;

                        rot.x = (mr[1] + mr[4]) / (4 * rot.y);
                        rot.w = (mr[8] - mr[2]) / (4 * rot.y);
                        rot.z = (mr[6] + mr[9]) / (4 * rot.y);
                    }
                    else {
                        rot.z = Math.sqrt(1 + mr[10] - mr[0] - mr[5]) / 2;

                        rot.x = (mr[8] + mr[2]) / (4 * rot.z);
                        rot.y = (mr[6] + mr[9]) / (4 * rot.z);
                        rot.w = (mr[1] - mr[4]) / (4 * rot.z);
                    }
                    break;

                case Orientation3D.EULER_ANGLES:
                    rot.y = Math.asin(-mr[2]);

                    if (mr[2] != 1 && mr[2] != -1) {
                        rot.x = Math.atan2(mr[6], mr[10]);
                        rot.z = Math.atan2(mr[1], mr[0]);
                    }
                    else {
                        rot.z = 0;
                        rot.x = Math.atan2(mr[4], mr[5]);
                    }
                    break;
            }

            var vec = new Vector.<Vector3D>([]);

            vec.push(pos);
            vec.push(rot);
            vec.push(scale);
            vec.push(skew);
            vec.push(perspective);

            return vec;
        }

        /**
         * Inverts the current matrix. An inverted matrix is the same size as the original
         * but performs the opposite transformation of the original matrix. For example, if
         * the original matrix has an object rotate around the x axis in one direction, the
         * inverse of the matrix will have the object rotate around the axis in the opposite
         * direction. Applying an inverted matrix to an object undoes the transformation
         * performed by the original matrix. If a matrix is multiplied by its inverse
         * matrix, the result is an identity matrix.
         * 
         * An inverse of a matrix can be used to divide one matrix by another. The way to
         * divide matrix A by matrix B is to multiply matrix A by the inverse of matrix B.
         * The inverse matrix can also be used with a camera space. When the camera moves in
         * the world space, the object in the world needs to move in the opposite direction
         * to transform from the world view to the camera or view space. For example, if the
         * camera moves closer, the objects becomes bigger. In other words, if the camera
         * moves down the world z axis, the object moves up world z axis.
         * 
         * The `invert()` method replaces the current matrix with an inverted matrix. If you
         * want to invert a matrix without altering the current matrix, first copy the
         * current matrix by using the clone() method and then apply the `invert()` method
         * to the copy.
         * 
         * The Matrix3D object must be invertible.

         * @return  Returns `true` if the matrix was successfully inverted.
         */
        public function invert():Boolean {
            var d = determinant;
            var invertable = Math.abs(d) > 0.00000000001;

            if (invertable) {
                d = 1 / d;

                var m11:Number = this._rawData[0];
                var m21:Number = this._rawData[4];
                var m31:Number = this._rawData[8];
                var m41:Number = this._rawData[12];
                var m12:Number = this._rawData[1];
                var m22:Number = this._rawData[5];
                var m32:Number = this._rawData[9];
                var m42:Number = this._rawData[13];
                var m13:Number = this._rawData[2];
                var m23:Number = this._rawData[6];
                var m33:Number = this._rawData[10];
                var m43:Number = this._rawData[14];
                var m14:Number = this._rawData[3];
                var m24:Number = this._rawData[7];
                var m34:Number = this._rawData[11];
                var m44:Number = this._rawData[15];

                this._rawData[0] = d * (m22 * (m33 * m44 - m43 * m34) - m32 * (m23 * m44 - m43 * m24) + m42 * (m23 * m34 - m33 * m24));
                this._rawData[1] = -d * (m12 * (m33 * m44 - m43 * m34) - m32 * (m13 * m44 - m43 * m14) + m42 * (m13 * m34 - m33 * m14));
                this._rawData[2] = d * (m12 * (m23 * m44 - m43 * m24) - m22 * (m13 * m44 - m43 * m14) + m42 * (m13 * m24 - m23 * m14));
                this._rawData[3] = -d * (m12 * (m23 * m34 - m33 * m24) - m22 * (m13 * m34 - m33 * m14) + m32 * (m13 * m24 - m23 * m14));
                this._rawData[4] = -d * (m21 * (m33 * m44 - m43 * m34) - m31 * (m23 * m44 - m43 * m24) + m41 * (m23 * m34 - m33 * m24));
                this._rawData[5] = d * (m11 * (m33 * m44 - m43 * m34) - m31 * (m13 * m44 - m43 * m14) + m41 * (m13 * m34 - m33 * m14));
                this._rawData[6] = -d * (m11 * (m23 * m44 - m43 * m24) - m21 * (m13 * m44 - m43 * m14) + m41 * (m13 * m24 - m23 * m14));
                this._rawData[7] = d * (m11 * (m23 * m34 - m33 * m24) - m21 * (m13 * m34 - m33 * m14) + m31 * (m13 * m24 - m23 * m14));
                this._rawData[8] = d * (m21 * (m32 * m44 - m42 * m34) - m31 * (m22 * m44 - m42 * m24) + m41 * (m22 * m34 - m32 * m24));
                this._rawData[9] = -d * (m11 * (m32 * m44 - m42 * m34) - m31 * (m12 * m44 - m42 * m14) + m41 * (m12 * m34 - m32 * m14));
                this._rawData[10] = d * (m11 * (m22 * m44 - m42 * m24) - m21 * (m12 * m44 - m42 * m14) + m41 * (m12 * m24 - m22 * m14));
                this._rawData[11] = -d * (m11 * (m22 * m34 - m32 * m24) - m21 * (m12 * m34 - m32 * m14) + m31 * (m12 * m24 - m22 * m14));
                this._rawData[12] = -d * (m21 * (m32 * m43 - m42 * m33) - m31 * (m22 * m43 - m42 * m23) + m41 * (m22 * m33 - m32 * m23));
                this._rawData[13] = d * (m11 * (m32 * m43 - m42 * m33) - m31 * (m12 * m43 - m42 * m13) + m41 * (m12 * m33 - m32 * m13));
                this._rawData[14] = -d * (m11 * (m22 * m43 - m42 * m23) - m21 * (m12 * m43 - m42 * m13) + m41 * (m12 * m23 - m22 * m13));
                this._rawData[15] = d * (m11 * (m22 * m33 - m32 * m23) - m21 * (m12 * m33 - m32 * m13) + m31 * (m12 * m23 - m22 * m13));
            }

            return invertable;
        }

        /**
         * A Number that determines whether a matrix is invertible.
         * 
         * A Matrix3D object must be invertible. You can use the `determinant` property to make
         * sure that a Matrix3D object is invertible. If determinant is zero, an inverse of
         * the matrix does not exist. For example, if an entire row or column of a matrix is
         * zero or if two rows or columns are equal, the determinant is zero. Determinant is
         * also used to solve a series of equations.
         * 
         * Only a square matrix, like the Matrix3D class, has a determinant.
         */
        public function get determinant():Number {
            mr = this.rawData;
            return 1 * ((mr[0] * mr[5] - mr[4] * mr[1]) * (mr[10] * mr[15] - mr[14] * mr[11])
                - (mr[0] * mr[9] - mr[8] * mr[1]) * (mr[6] * mr[15] - mr[14] * mr[7])
                + (mr[0] * mr[13] - mr[12] * mr[1]) * (mr[6] * mr[11] - mr[10] * mr[7])
                + (mr[4] * mr[9] - mr[8] * mr[5]) * (mr[2] * mr[15] - mr[14] * mr[3])
                - (mr[4] * mr[13] - mr[12] * mr[5]) * (mr[2] * mr[11] - mr[10] * mr[3])
                + (mr[8] * mr[13] - mr[12] * mr[9]) * (mr[2] * mr[7] - mr[6] * mr[3]));
        }

    }
}
