package
{
	import flash.display3D.Context3D;

	/**
	* A procedurally-generated quad
	* @author Jackson Dunstan
	*/
	public class Quad3D extends Shape3D
	{
		/** Positions of all quad's vertices */
		private static const POSITIONS:Vector.<Number> = new <Number>[
			-0.5, 0, -0.5,
			0.5, 0, -0.5,
			0.5, 0, 0.5,
			-0.5, 0, 0.5
		];
		
		/** Texture coordinates of all quad's vertices */
		private static const TEX_COORDS:Vector.<Number> = new <Number>[
			0, 0,
			1, 0,
			1, 1,
			0, 1
		];
		
		/** Triangles of all quads */
		private static const TRIS:Vector.<uint> = new <uint>[
			0, 2, 3, // bottom tri
			0, 1, 2  // top tri
		];
		
		/**
		* Make the quad
		* @param context Context to create the shape in
		* @param posX X position of the shape
		* @param posY Y position of the shape
		* @param posZ Z position of the shape
		* @param scaleX X scale of the shape
		* @param scaleY Y scale of the shape
		* @param scaleZ Z scale of the shape
		*/
		public function Quad3D(
			context:Context3D,
			posX:Number=0, posY:Number=0, posZ:Number=0,
			scaleX:Number=1, scaleY:Number=1, scaleZ:Number=1
		)
		{
			super(context, POSITIONS, TEX_COORDS, TRIS, posX, posY, posZ, scaleX, scaleY, scaleZ);
		}
	}
}
