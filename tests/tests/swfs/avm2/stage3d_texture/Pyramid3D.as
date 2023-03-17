package
{
	import flash.display3D.Context3D;

	/**
	* A procedurally-generated pyramid
	* @author Jackson Dunstan
	*/
	public class Pyramid3D extends Shape3D
	{
		/** Positions of all pyramids' vertices */
		private static const POSITIONS:Vector.<Number> = new <Number>[
			// back face
			-0.5, -0.5, -0.5,
			0, 0.5, 0,
			0.5, -0.5, -0.5,
			
			// front face
			-0.5, -0.5, 0.5,
			0, 0.5, 0,
			0.5, -0.5, 0.5,
			
			// left face
			-0.5, -0.5, -0.5,
			0, 0.5, 0,
			-0.5, -0.5, 0.5,
			
			// right face
			0.5, -0.5, -0.5,
			0, 0.5, 0,
			0.5, -0.5, 0.5,
			
			// bottom face - bottom tri
			-0.5, -0.5, 0.5,
			-0.5, -0.5, -0.5,
			0.5, -0.5, 0.5,
			// bottom face - top tri
			-0.5, -0.5, -0.5,
			0.5, -0.5, -0.5,
			0.5, -0.5, 0.5
		];
		
		/** Texture coordinates of all pyramids' vertices */
		private static const TEX_COORDS:Vector.<Number> = new <Number>[
			// back face
			1, 1,
			0.5, 0,
			0, 1,
			
			// front face
			0, 1,
			0.5, 0,
			1, 1,
			
			// left face
			0, 1,
			0.5, 0,
			1, 1,
			
			// right face
			1, 1,
			0.5, 0,
			0, 1,
			
			// bottom face - bottom tri
			0, 0,
			0, 1,
			1, 0,
			// bottom face - top tri
			0, 1,
			1, 1,
			1, 0
		];
		
		/** Triangles of all pyramids */
		private static const TRIS:Vector.<uint> = new <uint>[
			2, 1, 0,    // back face
			3, 4, 5,    // front face
			6, 7, 8,    // left face
			11, 10, 9,  // right face
			14, 13, 12, // bottom face - bottom tri
			17, 16, 15  // bottom face - top tri
		];
		
		/**
		* Make the pyramid
		* @param context Context to create the shape in
		* @param posX X position of the shape
		* @param posY Y position of the shape
		* @param posZ Z position of the shape
		* @param scaleX X scale of the shape
		* @param scaleY Y scale of the shape
		* @param scaleZ Z scale of the shape
		*/
		public function Pyramid3D(
			context:Context3D,
			posX:Number=0, posY:Number=0, posZ:Number=0,
			scaleX:Number=1, scaleY:Number=1, scaleZ:Number=1
		)
		{
			super(context, POSITIONS, TEX_COORDS, TRIS, posX, posY, posZ, scaleX, scaleY, scaleZ);
		}
	}
}
