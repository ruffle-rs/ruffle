package {
	import flash.display.ShaderData;
	import flash.utils.ByteArray;
	import flash.display.Shader;
	import flash.utils.getQualifiedClassName;

	public class Test {

		[Embed(source = "shader.pbj", mimeType="application/octet-stream")]
		public static var SHADER_BYTES: Class;
		
		public function Test() {
			var shader: ByteArray = new SHADER_BYTES();
			var data = new ShaderData(shader);
			trace(data);
			dumpObject(data);
		}
	
		private function dumpObject(obj: Object, prefix: String = "") {
			var keys = [];
			for (var k in obj) {
				keys.push(k)
			}
			keys.sort();
			for each (var key in keys) {
				trace(prefix + key + ": " + obj[key] + " (" + getQualifiedClassName(obj[key])+ ")");
				dumpObject(obj[key], prefix + " ");
			}
		}
	}
}