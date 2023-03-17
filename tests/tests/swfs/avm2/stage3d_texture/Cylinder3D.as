package
{
	import flash.display3D.Context3D;
	
	/**
	* A procedurally-generated cylinder
	* @author Jackson Dunstan
	*/
	public class Cylinder3D extends Shape3D
	{
		/** Minimum number of sides any cylinder can have */
		public static const MIN_SIDES:uint = 3;
		
		/**
		* Procedurally generate the cylinder
		* @param sides Number of sides of the cylinder. Clamped to at least
		*              MIN_SIDES. Increasing this will increase the smoothness of the cylinder at
		*              the cost of generating more vertices and triangles.
		* @param context Context to create the shape in
		* @param posX X position of the shape
		* @param posY Y position of the shape
		* @param posZ Z position of the shape
		* @param scaleX X scale of the shape
		* @param scaleY Y scale of the shape
		* @param scaleZ Z scale of the shape 
		*/
		public function Cylinder3D(
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
			const stepU:Number = 1.0 / sides;
			const verticesPerCircle:uint = sides + 1;
			const trisPerCap:uint = sides - 2;
			const firstSidePos:uint = verticesPerCircle+verticesPerCircle;
			
			var posIndex:uint;
			var texCoordIndex:uint;
			var triIndex:uint;
			
			var positions:Vector.<Number> = new Vector.<Number>(verticesPerCircle*12);
			var texCoords:Vector.<Number> = new Vector.<Number>(verticesPerCircle*8);
			var tris:Vector.<uint> = new Vector.<uint>((trisPerCap + trisPerCap + sides + sides)*3);
			
			var curTheta:Number = 0;
			var halfCosThetas:Vector.<Number> = new Vector.<Number>(verticesPerCircle);
			var halfSinThetas:Vector.<Number> = new Vector.<Number>(verticesPerCircle);
			for (var i:uint; i < verticesPerCircle; ++i)
			{
				halfCosThetas[i] = Math.cos(curTheta) * 0.5;
				halfSinThetas[i] = Math.sin(curTheta) * 0.5;
				curTheta += stepTheta;
			}
			
			// Top cap
			for (i = 0; i < verticesPerCircle; ++i)
			{
				positions[posIndex++] = halfCosThetas[i];
				positions[posIndex++] = 0.5;
				positions[posIndex++] = halfSinThetas[i];
				
				texCoords[texCoordIndex++] = halfCosThetas[i]+0.5;
				texCoords[texCoordIndex++] = halfSinThetas[i] + 0.5;
			}
			for (i = 0; i < trisPerCap; ++i)
			{
				tris[triIndex++] = 0;
				tris[triIndex++] = i+1;
				tris[triIndex++] = i+2;
			}
			
			// Bottom cap
			for (i = 0; i < verticesPerCircle; ++i)
			{
				positions[posIndex++] = halfCosThetas[i];
				positions[posIndex++] = -0.5;
				positions[posIndex++] = halfSinThetas[i];
				
				texCoords[texCoordIndex++] = halfCosThetas[i]+0.5;
				texCoords[texCoordIndex++] = -halfSinThetas[i] + 0.5;
			}
			for (i = 0; i < trisPerCap; ++i)
			{
				tris[triIndex++] = verticesPerCircle+i+2;
				tris[triIndex++] = verticesPerCircle+i+1;
				tris[triIndex++] = verticesPerCircle;
			}
			
			// Top cap (for the sides)
			var curU:Number = 1;
			for (i = 0; i < verticesPerCircle; ++i)
			{
				positions[posIndex++] = halfCosThetas[i];
				positions[posIndex++] = 0.5;
				positions[posIndex++] = halfSinThetas[i];
				
				texCoords[texCoordIndex++] = curU;
				texCoords[texCoordIndex++] = 0;
				
				curU -= stepU;
			}
			
			// Bottom cap (for the sides)
			curU = 1;
			for (i = 0; i < verticesPerCircle; ++i)
			{
				positions[posIndex++] = halfCosThetas[i];
				positions[posIndex++] = -0.5;
				positions[posIndex++] = halfSinThetas[i];
				
				texCoords[texCoordIndex++] = curU;
				texCoords[texCoordIndex++] = 1;
				
				curU -= stepU;
			}
			// Sides (excep the last quad)
			for (i = 0; i < sides; ++i)
			{
				// Top tri
				tris[triIndex++] = firstSidePos+verticesPerCircle+i+1; // bottom-right
				tris[triIndex++] = firstSidePos+i+1; // top-right
				tris[triIndex++] = firstSidePos+i; // top-left
				
				// Bottom tri
				tris[triIndex++] = firstSidePos+verticesPerCircle+i; // bottom-left
				tris[triIndex++] = firstSidePos+verticesPerCircle+i+1; // bottom-right
				tris[triIndex++] = firstSidePos+i; // top-left
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
			const trisPerCap:uint = sides - 2;
			return trisPerCap + trisPerCap + sides + sides;
		}
	}
}
