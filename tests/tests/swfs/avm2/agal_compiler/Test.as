package {
	import flash.utils.ByteArray;
	public class Test {
				
		private function readBytes(data: ByteArray): String {
			var out = new Array();
			data.position = 0;
    
			for (var i = 0; i < data.length; i++) {
				out.push(data.readUnsignedByte())
			}
			return out;
		}
	
		function dumpShader(text:String, code:ByteArray) {
			// Output in a format that we can easily copy-paste into a Rust file
			for each (var line in text.split("\n")) {
				trace("// " + line);
			}
			trace("[" + readBytes(code) + "]")
		}
	
		public function Test() {
			var vertexShaders = [
				"m44 op, va0, vc0    \n" +    // 4x4 matrix transform 
				"mov v0, va1", //copy color to varying variable v0
			
				"mov op, va0    \n" +    //copy position to output 
				"mov v0, va1" //copy color to varying variable v0
			];
			
			var fragmentShaders = [
				"mov oc, v0", //Set the output color to the value interpolated from the three triangle vertices 
			]
			
			trace("Vertex shaders:");
			for (var i = 0; i < vertexShaders.length; i++) {
				var vertexAssembler = new AGALMiniAssembler();
				vertexAssembler.assemble( "vertex", vertexShaders[i], 1, false );
				dumpShader(vertexShaders[i], vertexAssembler.agalcode);
				trace();
			}
			trace();
		
			trace("Fragment shaders:");
			for (var j = 0; j < fragmentShaders.length; j++) {
				var fragmentAssembler = new AGALMiniAssembler();
				fragmentAssembler.assemble( "fragment", fragmentShaders[j], 1, false );
				dumpShader(fragmentShaders[j], fragmentAssembler.agalcode);
				trace();
			}
		}
	}
}