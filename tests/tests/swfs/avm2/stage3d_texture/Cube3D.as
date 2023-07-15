package
{
	import flash.display3D.Context3D;

	/**
	* A procedurally-generated cube
	* @author Jackson Dunstan
	*/
	public class Cube3D extends Shape3D
	{
		/** Positions of all cubes' vertices */
		private static const POSITIONS:Vector.<Number> = new <Number>[
			// back face - bottom tri
			-0.5, -0.5, -0.5,
			-0.5, 0.5, -0.5,
			0.5, -0.5, -0.5,
			// back face - top tri
			-0.5, 0.5, -0.5,
			0.5, 0.5, -0.5,
			0.5, -0.5, -0.5,
			
			// front face - bottom tri
			-0.5, -0.5, 0.5,
			-0.5, 0.5, 0.5,
			0.5, -0.5, 0.5,
			// front face - top tri
			-0.5, 0.5, 0.5,
			0.5, 0.5, 0.5,
			0.5, -0.5, 0.5,
			
			// left face - bottom tri
			-0.5, -0.5, -0.5,
			-0.5, 0.5, -0.5,
			-0.5, -0.5, 0.5,
			// left face - top tri
			-0.5, 0.5, -0.5,
			-0.5, 0.5, 0.5,
			-0.5, -0.5, 0.5,
			
			// right face - bottom tri
			0.5, -0.5, -0.5,
			0.5, 0.5, -0.5,
			0.5, -0.5, 0.5,
			// right face - top tri
			0.5, 0.5, -0.5,
			0.5, 0.5, 0.5,
			0.5, -0.5, 0.5,
			
			// bottom face - bottom tri
			-0.5, -0.5, 0.5,
			-0.5, -0.5, -0.5,
			0.5, -0.5, 0.5,
			// bottom face - top tri
			-0.5, -0.5, -0.5,
			0.5, -0.5, -0.5,
			0.5, -0.5, 0.5,
			
			// top face - bottom tri
			-0.5, 0.5, 0.5,
			-0.5, 0.5, -0.5,
			0.5, 0.5, 0.5,
			// top face - top tri
			-0.5, 0.5, -0.5,
			0.5, 0.5, -0.5,
			0.5, 0.5, 0.5
		];
		
		/** Texture coordinates of all cubes' vertices */
		private static const TEX_COORDS:Vector.<Number> = new <Number>[
			// back face - bottom tri
			1, 1,
			1, 0,
			0, 1,
			// back face - top tri
			1, 0,
			0, 0,
			0, 1,
			
			// front face - bottom tri
			0, 1,
			0, 0,
			1, 1,
			// front face - top tri
			0, 0,
			1, 0,
			1, 1,
			
			// left face - bottom tri
			0, 1,
			0, 0,
			1, 1,
			// left face - top tri
			0, 0,
			1, 0,
			1, 1,
			
			// right face - bottom tri
			1, 1,
			1, 0,
			0, 1,
			// right face - top tri
			1, 0,
			0, 0,
			0, 1,
			
			// bottom face - bottom tri
			0, 0,
			0, 1,
			1, 0,
			// bottom face - top tri
			0, 1,
			1, 1,
			1, 0,
			
			// top face - bottom tri
			0, 1,
			0, 0,
			1, 1,
			// top face - top tri
			0, 0,
			1, 0,
			1, 1
		];
		
		/** Triangles of all cubes */
		private static const TRIS:Vector.<uint> = new <uint>[
			2, 1, 0,    // back face - bottom tri
			5, 4, 3,    // back face - top tri
			6, 7, 8,    // front face - bottom tri
			9, 10, 11,  // front face - top tri
			12, 13, 14, // left face - bottom tri
			15, 16, 17, // left face - top tri
			20, 19, 18, // right face - bottom tri
			23, 22, 21, // right face - top tri
			26, 25, 24, // bottom face - bottom tri
			29, 28, 27, // bottom face - top tri
			30, 31, 32, // top face - bottom tri
			33, 34, 35  // top face - bottom tri
		];
		
		/**
		* Make the cube
		* @param context Context to create the shape in
		* @param posX X position of the shape
		* @param posY Y position of the shape
		* @param posZ Z position of the shape
		* @param scaleX X scale of the shape
		* @param scaleY Y scale of the shape
		* @param scaleZ Z scale of the shape
		*/
		public function Cube3D(
			context:Context3D,
			posX:Number=0, posY:Number=0, posZ:Number=0,
			scaleX:Number=1, scaleY:Number=1, scaleZ:Number=1
		)
		{
			super(context, POSITIONS, TEX_COORDS, TRIS, posX, posY, posZ, scaleX, scaleY, scaleZ);
		}
	}
}
