package
{
	import flash.display3D.Context3D;
	import flash.geom.Matrix3D;
	import flash.display3D.IndexBuffer3D;
	import flash.display3D.VertexBuffer3D;
	
	/**
	* Base class of procedurally-generated 3D shapes
	* @author Jackson Dunstan
	*/
	public class Shape3D
	{
		/** Positions of the vertices of the sphere */
		public var positions:VertexBuffer3D;
		
		/** Texture coordinates of the vertices of the sphere */
		public var texCoords:VertexBuffer3D;
		
		/** Triangles of the sphere */
		public var tris:IndexBuffer3D;
		
		/** Matrix transforming the sphere from model space to world space */
		public var modelToWorld:Matrix3D;
		
		/**
		* Make the shape
		* @param context Context to create the shape in
		* @param positions Positions of the vertices of the shape
		* @param texCoords Texture coordinates of the vertices of the shape
		* @param tris Tris indexing the positions and texture coordinates of the shape
		* @param posX X position of the shape
		* @param posY Y position of the shape
		* @param posZ Z position of the shape
		* @param scaleX X scale of the shape
		* @param scaleY Y scale of the shape
		* @param scaleZ Z scale of the shape
		*/
		public function Shape3D(
			context:Context3D,
			positions:Vector.<Number>,
			texCoords:Vector.<Number>,
			tris:Vector.<uint>,
			posX:Number=0, posY:Number=0, posZ:Number=0,
			scaleX:Number=1, scaleY:Number=1, scaleZ:Number=1
		)
		{
			// Make the model->world transformation matrix to position and scale the sphere
			modelToWorld = new Matrix3D(
				new <Number>[
					scaleX, 0,      0,      posX,
					0,      scaleY, 0,      posY,
					0,      0,      scaleZ, posZ,
					0,      0,      0,      1
				]
			);
			
			// Create vertex and index buffers
			this.positions = context.createVertexBuffer(positions.length/3, 3);
			this.positions.uploadFromVector(positions, 0, positions.length/3);
			this.texCoords = context.createVertexBuffer(texCoords.length/2, 2);
			this.texCoords.uploadFromVector(texCoords, 0, texCoords.length/2);
			this.tris = context.createIndexBuffer(tris.length);
			this.tris.uploadFromVector(tris, 0, tris.length);
		}
		
		public function dispose(): void
		{
			this.positions.dispose();
			this.texCoords.dispose();
			this.tris.dispose();
		}
	}
}
