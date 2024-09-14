package
{
	import flash.geom.Matrix3D;
	import flash.geom.Vector3D;
	
	/**
	 *   A 3D camera using perspective projection
	 *   @author Jackson Dunstan
	 */
	public class Camera3D
	{
		/** Minimum distance the near plane can be */
		public static const MIN_NEAR_DISTANCE:Number = 0.001;
		
		/** Minimum distance between the near and far planes */
		public static const MIN_PLANE_SEPARATION:Number = 0.001;
		
		/** Position of the camera */
		private var __position:Vector3D;
		
		/** What the camera is looking at */
		private var __target:Vector3D;
		
		/** Direction that is "up" */
		private var __upDir:Vector3D;
		
		/** Direction that is "up" */
		private var __realUpDir:Vector3D;
		
		/** Near clipping plane distance */
		private var __near:Number;
		
		/** Far clipping plane distance */
		private var __far:Number;
		
		/** Aspect ratio of the camera lens */
		private var __aspect:Number;
		
		/** Vertical field of view */
		private var __vFOV:Number;
		
		/** World->View transformation */
		private var __worldToView:Matrix3D;
		
		/** View->Clip transformation */
		private var __viewToClip:Matrix3D;
		
		/** World->Clip transformation */
		private var __worldToClip:Matrix3D;
		
		/** Direction the camera is pointing */
		private var __viewDir:Vector3D;
		
		/** Magnitude of the view direction */
		private var __viewDirMag:Number;
		
		/** Direction to the right of where the camera is pointing */
		private var __rightDir:Vector3D;
		
		/** A temporary matrix for use during world->view calculation */
		private var __tempWorldToViewMatrix:Matrix3D;
		
		/** Frustum planes: left, right, bottom, top, near, far */
		private var __frustumPlanes:Vector.<Vector3D> = new <Vector3D>[
			new Vector3D(),
			new Vector3D(),
			new Vector3D(),
			new Vector3D(),
			new Vector3D(),
			new Vector3D()
		];
		
		/**
		*   Make the camera
		*   @param near Distance to the near clipping plane. Capped to MIN_NEAR_DISTANCE.
		*   @param far Distance to the far clipping plane. Must be MIN_PLANE_SEPARATION greater than near.
		*   @param aspect Aspect ratio of the camera lens
		*   @param vFOV Vertical field of view
		*   @param positionX X component of the camera's position
		*   @param positionY Y component of the camera's position
		*   @param positionZ Z component of the camera's position
		*   @param targetX X component of the point the camera is aiming at
		*   @param targetY Y component of the point the camera is aiming at
		*   @param targetZ Z component of the point the camera is aiming at
		*   @param upDirX X component of the direction considered to be "up"
		*   @param upDirX X component of the direction considered to be "up"
		*   @param upDirY Y component of the direction considered to be "up"
		*/
		public function Camera3D(
			near:Number,
			far:Number,
			aspect:Number,
			vFOV:Number,
			positionX:Number,
			positionY:Number,
			positionZ:Number,
			targetX:Number,
			targetY:Number,
			targetZ:Number,
			upDirX:Number,
			upDirY:Number,
			upDirZ:Number
		)
		{
			if (near < MIN_NEAR_DISTANCE)
			{
				near = MIN_NEAR_DISTANCE;
			}
			
			if (far < near+MIN_PLANE_SEPARATION)
			{
				far = near + MIN_PLANE_SEPARATION;
			}
			
			__near = near;
			__far = far;
			__aspect = aspect;
			__vFOV = vFOV;
			__position = new Vector3D(positionX, positionY, positionZ);
			__target = new Vector3D(targetX, targetY, targetZ);
			__upDir = new Vector3D(upDirX, upDirY, upDirZ);
			__upDir.normalize();
			
			__viewDir = new Vector3D();
			__rightDir = new Vector3D();
			__realUpDir = new Vector3D();
			__tempWorldToViewMatrix = new Matrix3D();
			
			__worldToView = new Matrix3D();
			__viewToClip = new Matrix3D();
			__worldToClip = new Matrix3D();
			
			updateWorldToView();
			updateViewToClip();
			updateWorldToClip();
		}
		
		/**
		*   Get the world->clip transformation
		*   @return The world->clip transformation
		*/
		public function get worldToClipMatrix(): Matrix3D
		{
			return __worldToClip;
		}
		
		/**
		*   Get the camera's position in the X
		*   @return The camera's position in the X
		*/
		public function get positionX(): Number
		{
			return __position.x;
		}
		
		/**
		*   Set the camera's position in the X
		*   @param x The camera's position in the X
		*/
		public function set positionX(x:Number): void
		{
			__position.x = x;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Get the camera's position in the Y
		*   @return The camera's position in the Y
		*/
		public function get positionY(): Number
		{
			return __position.y;
		}
		
		/**
		*   Set the camera's position in the Y
		*   @param y The camera's position in the Y
		*/
		public function set positionY(y:Number): void
		{
			__position.y = y;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Get the camera's position in the Z
		*   @return The camera's position in the Z
		*/
		public function get positionZ(): Number
		{
			return __position.z;
		}
		
		/**
		*   Set the camera's position in the Z
		*   @param z The camera's position in the Z
		*/
		public function set positionZ(z:Number): void
		{
			__position.z = z;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Set the camera's position
		*   @param x The camera's position in the X
		*   @param y The camera's position in the Y
		*   @param z The camera's position in the Z
		*/
		public function setPositionValues(x:Number, y:Number, z:Number): void
		{
			__position.x = x;
			__position.y = y;
			__position.z = z;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Get the camera's target in the X
		*   @return The camera's target in the X
		*/
		public function get targetX(): Number
		{
			return __target.x;
		}
		
		/**
		*   Set the camera's target in the X
		*   @param x The camera's target in the X
		*/
		public function set targetX(x:Number): void
		{
			__target.x = x;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Get the camera's target in the Y
		*   @return The camera's target in the Y
		*/
		public function get targetY(): Number
		{
			return __target.y;
		}
		
		/**
		*   Set the camera's target in the Y
		*   @param y The camera's target in the Y
		*/
		public function set targetY(y:Number): void
		{
			__target.y = y;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Get the camera's target in the Z
		*   @return The camera's target in the Z
		*/
		public function get targetZ(): Number
		{
			return __target.z;
		}
		
		/**
		*   Set the camera's target in the Z
		*   @param z The camera's target in the Z
		*/
		public function set targetZ(z:Number): void
		{
			__target.z = z;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Set the camera's target
		*   @param x The camera's target in the X
		*   @param y The camera's target in the Y
		*   @param z The camera's target in the Z
		*/
		public function setTargetValues(x:Number, y:Number, z:Number): void
		{
			__target.x = x;
			__target.y = y;
			__target.z = z;
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Get the near clipping distance
		*   @return The near clipping distance
		*/
		public function get near(): Number
		{
			return __near;
		}
		
		/**
		*   Set the near clipping distance
		*   @param near The near clipping distance
		*/
		public function set near(near:Number): void
		{
			__near = near;
			updateViewToClip();
			updateWorldToClip();
		}
		
		/**
		*   Get the far clipping distance
		*   @return The far clipping distance
		*/
		public function get far(): Number
		{
			return __far;
		}
		
		/**
		*   Set the far clipping distance
		*   @param far The far clipping distance
		*/
		public function set far(far:Number): void
		{
			__far = far;
			updateViewToClip();
			updateWorldToClip();
		}
		
		/**
		*   Get the vertical field of view angle
		*   @return The vertical field of view angle
		*/
		public function get vFOV(): Number
		{
			return __vFOV;
		}
		
		/**
		*   Set the vertical field of view angle
		*   @param vFOV The vertical field of view angle
		*/
		public function set vFOV(vFOV:Number): void
		{
			__vFOV = vFOV;
			updateViewToClip();
			updateWorldToClip();
		}
		
		/**
		*   Get the aspect ratio
		*   @return The aspect ratio
		*/
		public function get aspect(): Number
		{
			return __aspect;
		}
		
		/**
		*   Set the aspect ratio
		*   @param aspect The aspect ratio
		*/
		public function set aspect(aspect:Number): void
		{
			__aspect = aspect;
			updateViewToClip();
			updateWorldToClip();
		}
		
		/**
		*   Move the camera toward the target
		*   @param units Number of units to move forward
		*/
		public function moveForward(units:Number): void
		{
			moveAlongAxis(units, __viewDir);
		}
		
		/**
		*   Move the camera away from the target
		*   @param units Number of units to move backward
		*/
		public function moveBackward(units:Number): void
		{
			moveAlongAxis(-units, __viewDir);
		}
		
		/**
		*   Move the camera right
		*   @param units Number of units to move right
		*/
		public function moveRight(units:Number): void
		{
			moveAlongAxis(units, __rightDir);
		}
		
		/**
		*   Move the camera left
		*   @param units Number of units to move left
		*/
		public function moveLeft(units:Number): void
		{
			moveAlongAxis(-units, __rightDir);
		}
		
		/**
		*   Move the camera up
		*   @param units Number of units to move up
		*/
		public function moveUp(units:Number): void
		{
			moveAlongAxis(units, __upDir);
		}
		
		/**
		*   Move the camera down
		*   @param units Number of units to move down
		*/
		public function moveDown(units:Number): void
		{
			moveAlongAxis(-units, __upDir);
		}
		
		/**
		*   Move the camera right toward the target
		*   @param units Number of units to move right
		*   @param axis Axis to move along
		*/
		private function moveAlongAxis(units:Number, axis:Vector3D): void
		{
			var delta:Vector3D = axis.clone();
			delta.scaleBy(units);
			
			var newPos:Vector3D = __position.add(delta);
			setPositionValues(newPos.x, newPos.y, newPos.z);
			
			var newTarget:Vector3D = __target.add(delta);
			setTargetValues(newTarget.x, newTarget.y, newTarget.z);
		}
		
		/**
		*   Yaw the camera left/right
		*   @param numDegrees Number of degrees to yaw. Positive is clockwise,
		*                     negative is counter-clockwise. If NaN, this
		*                     function does nothing.
		*/
		public function yaw(numDegrees:Number): void
		{
			rotate(numDegrees, __realUpDir);
		}
		
		/**
		*   Pitch the camera up/down
		*   @param numDegrees Number of degrees to pitch. Positive is clockwise,
		*                     negative is counter-clockwise. If NaN, this
		*                     function does nothing.
		*/
		public function pitch(numDegrees:Number): void
		{
			rotate(numDegrees, __rightDir);
		}
		
		/**
		*   Roll the camera left/right
		*   @param numDegrees Number of degrees to roll. Positive is clockwise,
		*                     negative is counter-clockwise. If NaN, this
		*                     function does nothing.
		*/
		public function roll(numDegrees:Number): void
		{
			if (isNaN(numDegrees))
			{
				return;
			}
			
			// Make positive and negative make sense
			numDegrees = -numDegrees;
			
			var rotMat:Matrix3D = new Matrix3D();
			rotMat.appendRotation(numDegrees, __viewDir);
			
			__upDir = rotMat.transformVector(__upDir);
			__upDir.normalize();
			
			updateWorldToView();
			updateWorldToClip();
		}
		
		/**
		*   Rotate the camera about an axis
		*   @param numDegrees Number of degrees to rotate. Positive is clockwise,
		*                     negative is counter-clockwise. If NaN, this
		*                     function does nothing.
		*   @param axis Axis of rotation
		*/
		private function rotate(numDegrees:Number, axis:Vector3D): void
		{
			if (isNaN(numDegrees))
			{
				return;
			}
			
			// Make positive and negative make sense
			numDegrees = -numDegrees;
			
			var rotMat:Matrix3D = new Matrix3D();
			rotMat.appendRotation(numDegrees, axis);
			
			var rotatedViewDir:Vector3D = rotMat.transformVector(__viewDir);
			rotatedViewDir.scaleBy(__viewDirMag);
			
			var newTarget:Vector3D = __position.add(rotatedViewDir);
			
			setTargetValues(newTarget.x, newTarget.y, newTarget.z);
		}
		
		/**
		*   Get the distance between a point and a plane
		*   @param point Point to get the distance between
		*   @param plane Plane to get the distance between
		*   @return The distance between the given point and plane
		*/
		private static function pointPlaneDistance(point:Vector3D, plane:Vector3D): Number
		{
			// plane distance + (point [dot] plane)
			return (plane.w + (point.x*plane.x + point.y*plane.y + point.z*plane.z));
		}
		
		/**
		*   Check if a point is in the viewing frustum
		*   @param point Point to check
		*   @return If the given point is in the viewing frustum
		*/
		public function isPointInFrustum(point:Vector3D): Boolean
		{
			for each (var plane:Vector3D in __frustumPlanes)
			{
				if (pointPlaneDistance(point, plane) < 0)
				{
					return false;
				}
			}
			return true;
		}
		
		/**
		*   Check if a sphere is in the viewing frustum
		*   @param sphere Sphere to check. XYZ are the center, W is the radius.
		*   @return If any part of the given sphere is in the viewing frustum
		*/
		public function isSphereInFrustum(sphere:Vector3D): Boolean
		{
			// Test all extents of the sphere 
			var minusRadius:Number = -sphere.w;
			for each (var plane:Vector3D in __frustumPlanes)
			{
				if (pointPlaneDistance(sphere, plane) < minusRadius)
				{
					return false;
				}
			}
			return true;
		}
		
		/**
		*   Update the world->view matrix
		*/
		private function updateWorldToView(): void
		{
			// viewDir = target - position
			var viewDir:Vector3D = __viewDir;
			viewDir.x = __target.x - __position.x;
			viewDir.y = __target.y - __position.y;
			viewDir.z = __target.z - __position.z;
			__viewDirMag = __viewDir.normalize();
			
			// Up is already normalized
			var upDir:Vector3D = __upDir;
			
			// rightDir = viewDir X upPrime
			var rightDir:Vector3D = __rightDir;
			rightDir.x = viewDir.y*upDir.z - viewDir.z*upDir.y;
			rightDir.y = viewDir.z*upDir.x - viewDir.x*upDir.z;
			rightDir.z = viewDir.x*upDir.y - viewDir.y*upDir.x;
			
			// realUpDir = rightDir X viewDir
			var realUpDir:Vector3D = __realUpDir;
			realUpDir.x = rightDir.y*viewDir.z - rightDir.z*viewDir.y;
			realUpDir.y = rightDir.z*viewDir.x - rightDir.x*viewDir.z;
			realUpDir.z = rightDir.x*viewDir.y - rightDir.y*viewDir.x;
			
			// Translation by -position
			var rawData:Vector.<Number> = __worldToView.rawData;
			rawData[0] = 1;
			rawData[1] = 0;
			rawData[2] = 0;
			rawData[3] = -__position.x;
			rawData[4] = 0;
			rawData[5] = 1;
			rawData[6] = 0;
			rawData[7] = -__position.y;
			rawData[8] = 0;
			rawData[9] = 0;
			rawData[10] = 1;
			rawData[11] = -__position.z;
			rawData[12] = 0;
			rawData[13] = 0;
			rawData[14] = 0;
			rawData[15] = 1;
			__worldToView.rawData = rawData;
			
			// Look At matrix. Some parts of this are constant.
			rawData = __tempWorldToViewMatrix.rawData;
			rawData[0] = rightDir.x;
			rawData[1] = rightDir.y;
			rawData[2] = rightDir.z;
			rawData[3] = 0;
			rawData[4] = realUpDir.x;
			rawData[5] = realUpDir.y;
			rawData[6] = realUpDir.z;
			rawData[7] = 0;
			rawData[8] = -viewDir.x;
			rawData[9] = -viewDir.y;
			rawData[10] = -viewDir.z;
			rawData[11] = 0;
			rawData[12] = 0;
			rawData[13] = 0;
			rawData[14] = 0;
			rawData[15] = 1;
			__tempWorldToViewMatrix.rawData = rawData;
			
			__worldToView.prepend(__tempWorldToViewMatrix);
		}
		
		/**
		*   Update the view->clip matrix
		*/
		private function updateViewToClip(): void
		{
			var f:Number = 1.0 / Math.tan(__vFOV);
			__viewToClip.rawData = new <Number>[
				f / __aspect, 0,                               0,                                 0,
				0,            f,                               0,                                 0,
				0,            0, ((__far+__near)/(__near-__far)), ((2*__far*__near)/(__near-__far)),
				0,            0,                              -1,                                 0
			];
		}
		
		/**
		*   Update the world->clip matrix
		*/
		private function updateWorldToClip(): void
		{
			__worldToView.copyToMatrix3D(__worldToClip);
			__worldToClip.prepend(__viewToClip);
			
			var rawData:Vector.<Number> = __worldToClip.rawData;
			var plane:Vector3D;
			
			// left = row1 + row4
			plane = __frustumPlanes[0];
			plane.x = rawData[0] + rawData[12];
			plane.y = rawData[1] + rawData[13];
			plane.z = rawData[2] + rawData[14];
			plane.w = rawData[3] + rawData[15];
			
			// right = -row1 + row4
			plane = __frustumPlanes[1];
			plane.x = -rawData[0] + rawData[12];
			plane.y = -rawData[1] + rawData[13];
			plane.z = -rawData[2] + rawData[14];
			plane.w = -rawData[3] + rawData[15];
			
			// bottom = row2 + row4
			plane = __frustumPlanes[2];
			plane.x = rawData[4] + rawData[12];
			plane.y = rawData[5] + rawData[13];
			plane.z = rawData[6] + rawData[14];
			plane.w = rawData[7] + rawData[15];
			
			// top = -row2 + row4
			plane = __frustumPlanes[3];
			plane.x = -rawData[4] + rawData[12];
			plane.y = -rawData[5] + rawData[13];
			plane.z = -rawData[6] + rawData[14];
			plane.w = -rawData[7] + rawData[15];
			
			// near = row3 + row4
			plane = __frustumPlanes[4];
			plane.x = rawData[8] + rawData[12];
			plane.y = rawData[9] + rawData[13];
			plane.z = rawData[10] + rawData[14];
			plane.w = rawData[11] + rawData[15];
			
			// far = -row3 + row4
			plane = __frustumPlanes[5];
			plane.x = -rawData[8] + rawData[12];
			plane.y = -rawData[9] + rawData[13];
			plane.z = -rawData[10] + rawData[14];
			plane.w = -rawData[11] + rawData[15];
		}
	}
}
