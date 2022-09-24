/* Copyright (c) 2015 EL-EMENT saharan
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this
 * software and associated documentation  * files (the "Software"), to deal in the Software
 * without restriction, including without limitation the rights to use, copy,  * modify, merge,
 * publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to
 * whom the Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or
 * substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
 * INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
 * PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR
 * ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
package {
	import com.adobe.utils.AGALMiniAssembler;
	import com.element.oimo.ogsl.OGSL;
	import flash.display.*;
	import flash.display3D.*;
	import flash.display3D.textures.CubeTexture;
	import flash.events.*;
	import flash.geom.*;

	/**
	 * OGSL sample
	 * @author saharan
	 */
	[SWF(width = "512", height = "512", frameRate = "60")]
	public class Sample extends Sprite {
		private const WIDTH:int = 512;
		private const HEIGHT:int = 512;
		private var s3d:Stage3D;
		private var c3d:Context3D;
		private var ibuf:IndexBuffer3D;
		private var ogsl:OGSL;
		private var count:int;

		public function Sample() {
			if (stage) init();
			else addEventListener(Event.ADDED_TO_STAGE, init);
		}

		private function init(e:Event = null):void {
			removeEventListener(Event.ADDED_TO_STAGE, init);

			s3d = stage.stage3Ds[0];
			s3d.addEventListener(Event.CONTEXT3D_CREATE, onCreateContext3D);
			// Use STANDARD_CONSTRAINED or STANDARD profile in order to use AGAL2, otherwise you cannot use if...else statements.
			s3d.requestContext3D(Context3DRenderMode.AUTO, Context3DProfile.STANDARD_CONSTRAINED);
		}

		private function onCreateContext3D(e:Event):void {
			c3d = s3d.context3D;

			c3d.enableErrorChecking = true;
			c3d.configureBackBuffer(WIDTH, HEIGHT, 0);

			ogsl = new OGSL();
			ogsl.setLogFunction(function(s:String):void { // set log function
				trace(s);
			});
			ogsl.compile(OGSL_SOURCE);
			ogsl.setContext3D(c3d);

			createVertexBuffer();
			createIndexBuffer();
			createProgram();

			ogsl.setVertexConstantsFromMatrix("matrix", new Matrix3D());

			addEventListener(Event.ENTER_FRAME, loop);
		}

		private function createVertexBuffer():void {
			var vertexBuffer:VertexBuffer3D = c3d.createVertexBuffer(4, 3);
			vertexBuffer.uploadFromVector(Vector.<Number>([
				-1, -1, 0, // x, y, z
				 1, -1, 0,
				-1,  1, 0,
				 1,  1, 0,
			]), 0, 4);
			ogsl.setVertexBuffer("position", vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_3);
		}

		private function createIndexBuffer():void {
			ibuf = c3d.createIndexBuffer(6);
			ibuf.uploadFromVector(Vector.<uint>([
				0, 1, 2,
				1, 3, 2,
			]), 0, 6);
		}

		private function createProgram():void {
			var assembler:AGALMiniAssembler = new AGALMiniAssembler();

			var program:Program3D = c3d.createProgram();
			program.upload(
				assembler.assemble(Context3DProgramType.VERTEX, ogsl.getVertexAGAL(), 2),
				assembler.assemble(Context3DProgramType.FRAGMENT, ogsl.getFragmentAGAL(), 2)
			);
			c3d.setProgram(program);
			ogsl.setDefaultConstants(); // set hard-coded constants
		}

		private function loop(e:Event):void {
			count++;
			c3d.clear();
			var halfWidth:Number = WIDTH / 2;
			var halfHeight:Number = HEIGHT / 2;
			ogsl.setFragmentConstantsFromVector("mousePos", new <Number>[(mouseX - halfWidth) / halfWidth, (halfHeight - mouseY) / halfHeight]);
			ogsl.setFragmentConstantsFromNumber("time", count / 60);
			c3d.drawTriangles(ibuf);
			c3d.present();
		}
	}
}

// try editing this...
const OGSL_SOURCE:String = <![CDATA[

varying pos:vec2;

program vertex {
	attribute position:vec3;
	uniform matrix:mat4x4;

	function main():void {
		output = mul(matrix, vec4(position, 1));
		pos = position.xy;
	}
}

program fragment {
	uniform mousePos:vec2;
	uniform time:float;

	function main():void {
		var dist:float = distance(mousePos, pos);
		var brightness:float = max(1 - dist, 0);
		var theta:float = dist * 3 + time;
		var color:vec3 = vec3(sin(theta * 2.3), sin(theta * 2.5), sin(theta * 2.7));
		color = color * 0.5 + 0.5;
		output.rgb = color * round(brightness * 16) / 16;
	}
}

]]>
