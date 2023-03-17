package
{
	import flash.display3D.Context3D;
	
	/**
	* A procedurally-generated circle
	* @author Jackson Dunstan
	*/
	public class Circle3D extends Shape3D
	{
		/** Minimum number of sides of a circle */
		public static const MIN_SIDES:uint = 3;
		
		/**
		* Make the circle
		* @param sides Number of sides the circle has. Capped to MIN_SIDES.
		* @param context Context to create the shape in
		* @param posX X position of the shape
		* @param posY Y position of the shape
		* @param posZ Z position of the shape
		* @param scaleX X scale of the shape
		* @param scaleY Y scale of the shape
		* @param scaleZ Z scale of the shape
		*/
		public function Circle3D(
			sides:uint,
			context:Context3D,
			posX:Number=0, posY:Number=0, posZ:Number=0,
			scaleX:Number=1, scaleY:Number=1, scaleZ:Number=1
		)
		{
			// Cap sides
			if (sides < MIN_SIDES)
			{
				sides = MIN_SIDES;
			}
			
			const stepTheta:Number = (2.0*Math.PI) / sides;
			const numVertices:uint = sides + 1;
			const numTris:uint = sides - 2;
			
			var posIndex:uint;
			var texCoordIndex:uint;
			var triIndex:uint;
			
			var positions:Vector.<Number> = new Vector.<Number>(numVertices*3);
			var texCoords:Vector.<Number> = new Vector.<Number>(numVertices*2);
			var tris:Vector.<uint> = new Vector.<uint>(numTris * 3);
			
			var curTheta:Number = 0;
			for (var i:uint = 0; i < numVertices; ++i)
			{
				var cos:Number = Math.cos(curTheta) * 0.5;
				var sin:Number = Math.sin(curTheta) * 0.5;
				
				positions[posIndex++] = cos;
				positions[posIndex++] = 0;
				positions[posIndex++] = sin;
				
				texCoords[texCoordIndex++] = cos+0.5;
				texCoords[texCoordIndex++] = sin + 0.5;
				
				curTheta += stepTheta;
			}
			for (i = 0; i < numTris; ++i)
			{
				tris[triIndex++] = 0;
				tris[triIndex++] = i+1;
				tris[triIndex++] = i+2;
			}
				
			super(context, positions, texCoords, tris, posX, posY, posZ, scaleX, scaleY, scaleZ);
		}
		
		public static function computeNumTris(sides:uint): uint
		{
			// Cap sides
			if (sides < MIN_SIDES)
			{
				sides = MIN_SIDES;
			}
			return sides - 2;
		}
	}
}
