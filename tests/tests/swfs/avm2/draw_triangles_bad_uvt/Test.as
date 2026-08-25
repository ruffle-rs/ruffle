package {

	import flash.display.MovieClip;

	public class Test extends MovieClip {

		public function Test() {
			var vertices = new Vector.<Number>();
			var indices = new Vector.<int>();
			var uvt = new Vector.<Number>(1.0);
			this.graphics.drawTriangles(vertices, indices, uvt);

			vertices.push(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
			this.graphics.drawTriangles(vertices, indices);

			indices.push(0, 1, 2);
			this.graphics.drawTriangles(vertices, indices);

			try {
				this.graphics.drawTriangles(vertices, indices, uvt);
			}
			catch (e) {
				trace("Caught err: " + e);
			}
		}
	}

}
