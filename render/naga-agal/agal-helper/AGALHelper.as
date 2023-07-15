package {
	import flash.display3D.Context3DProgramType;
	import com.adobe.utils.AGALMiniAssembler;
	import flash.utils.ByteArray;

	public class AGALHelper {
		public function AGALHelper() {
			// Edit these with the AGAL you want to compile

			var vertexShader:AGALMiniAssembler = new AGALMiniAssembler();
			var vertexBytes = vertexShader.assemble(Context3DProgramType.VERTEX,
				"mov vt0, vc[va0.x+5]    \n" +
			    "mov vt1, vc[va1.y+6]    \n" +
			    "add op, vt0, vt1",
				2);
			trace("Vertex shader:");
			printArray(vertexBytes);

			var fragmentShader:AGALMiniAssembler = new AGALMiniAssembler();
			var fragmentBytes = vertexShader.assemble(Context3DProgramType.FRAGMENT,
				"mov oc, v0\n",
				2);

			trace("Fragment shader:");
			printArray(fragmentBytes);
		}

		private function printArray(data:ByteArray) {
			var out = "&[";
			data.position = 0;
			while (data.bytesAvailable != 0) {
				out += data.readUnsignedByte();
				out += ",";
			}
			out += "]";
			trace(out);
		}
	}
}