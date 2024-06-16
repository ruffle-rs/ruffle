package {
	import flash.display.ShaderData;
	import flash.utils.ByteArray;
	import flash.display.Shader;
	import flash.utils.getQualifiedClassName;
	import flash.display.ShaderInput;
	import flash.display.ShaderParameter;
	

	public class Test {

		[Embed(source = "shader.pbj", mimeType="application/octet-stream")]
		public static var SHADER_BYTES: Class;
		
		public function Test() {
			var shader: ByteArray = new SHADER_BYTES();
			var data = new ShaderData(shader);
			trace(data);
			dumpObject(data);
			trace("Setting size to null");
			data.size.value = null;
			trace("Size: " + data.size.value);
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
			if (obj is ShaderInput) {
				trace(prefix + "channels: " + obj.channels);
				trace(prefix + "height: " + obj.height);
				trace(prefix + "index: " + obj.index);
				trace(prefix + "height: " + obj.height);
				trace(prefix + "input: " + obj.input);
				trace(prefix + "width: " + obj.width);
			} else if (obj is ShaderParameter) {
				trace(prefix + "index: " + obj.index);
				trace(prefix + "type: " + obj.type);
				trace(prefix + "value: " + obj.value);
			}
		}
	}
}