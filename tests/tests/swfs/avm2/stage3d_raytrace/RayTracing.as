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
	import flash.utils.ByteArray;
	import flash.errors.EOFError;

	/**
	 * Real-time ray tracing demo
	 * @author saharan
	 */
	//[SWF(width = "640", height = "480", frameRate = "60")]
	public class RayTracing extends Sprite {
		private var stage3D:Stage3D;
		private var c3d:Context3D;
		private var indexBuffer:IndexBuffer3D;
		private var ogsl:OGSL;
		private var count:int;
		private var mx:Number;
		private var my:Number;
		private const WIDTH:int = 640;
		private const HEIGHT:int = 480;

		public function RayTracing() {
			if (stage) init();
			else addEventListener(Event.ADDED_TO_STAGE, init);
		}

		private function init(e:Event = null):void {
			removeEventListener(Event.ADDED_TO_STAGE, init);
			stage3D = stage.stage3Ds[0];
			stage3D.addEventListener(Event.CONTEXT3D_CREATE, onCreateContext3D);
			stage3D.requestContext3D(Context3DRenderMode.AUTO, Context3DProfile.STANDARD_CONSTRAINED);
		}

		private function onCreateContext3D(e:Event):void {
			c3d = stage3D.context3D;

			c3d.enableErrorChecking = true;
			c3d.configureBackBuffer(WIDTH, HEIGHT, 0);

			ogsl = new OGSL();
			//ogsl.setLogFunction(trace);
			ogsl.compile(OGSL_SOURCE);
			ogsl.setContext3D(c3d);

			createVertexBuffer();
			createIndexBuffer();
			createProgram();

			ogsl.setVertexConstantsFromMatrix("matrix", new Matrix3D());
			mx = mouseX;
			my = mouseY;

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
			indexBuffer = c3d.createIndexBuffer(6);
			indexBuffer.uploadFromVector(Vector.<uint>([
				0, 1, 2,
				1, 3, 2,
			]), 0, 6);
		}

		[Embed(source = "/cube0.jpg")]
		private static const CUBE_0:Class;
		[Embed(source = "/cube1.jpg")]
		private static const CUBE_1:Class;
		[Embed(source = "/cube2.jpg")]
		private static const CUBE_2:Class;
		[Embed(source = "/cube3.jpg")]
		private static const CUBE_3:Class;
		[Embed(source = "/cube4.jpg")]
		private static const CUBE_4:Class;
		[Embed(source = "/cube5.jpg")]
		private static const CUBE_5:Class;

		// To speed up the runtime in debug builds of Ruffle, I've
		// extracted the final AGAL output of the 'OGSL' class, and
		// included them here.
	
		[Embed(source = "/precompiled_vertex_shader.agal", mimeType="application/octet-stream")]
		private static const PRECOMPILED_VERTEX_SHADER:Class;
	
		[Embed(source = "/precompiled_fragment_shader.agal", mimeType="application/octet-stream")]
		private static const PRECOMPILED_FRAGMENT_SHADER:Class;

		private function createProgram():void {
			var assembler:AGALMiniAssembler = new AGALMiniAssembler();
			
			//var ogslvertexAssembly = assembler.assemble(Context3DProgramType.VERTEX, ogsl.getVertexAGAL(), 2);
			//var ogslfragmentAssembly = assembler.assemble(Context3DProgramType.FRAGMENT, ogsl.getFragmentAGAL(), 2);
			
			var vertexAssembly = new PRECOMPILED_VERTEX_SHADER();
			var fragmentAssembly = new PRECOMPILED_FRAGMENT_SHADER();
			
			vertexAssembly.endian = "littleEndian";
			fragmentAssembly.endian = "littleEndian";
			
			function toNumberArray(bArray:ByteArray) {
				var bytes = [];
				bArray.position = 0;
				while (bArray.position != bArray.length) {
					bytes.push(bArray.readUnsignedByte());
				}
				bArray.position = 0;
				return bytes;
			}
			
			/*trace("Vertex assembly: ");
			trace("[" + toNumberArray(vertexAssembly) + "]");
		
			trace("OGSL Vertex assembly: ");
			trace("[" + toNumberArray(ogslvertexAssembly) + "]");
		
			trace("Fragment assembly: ");
			trace("[" + toNumberArray(fragmentAssembly) + "]");
		
			trace("OGSL Fragment assembly: ");
			trace("[" + toNumberArray(ogslfragmentAssembly) + "]");*/

			var program:Program3D = c3d.createProgram();
			program.upload(vertexAssembly, fragmentAssembly);
			c3d.setProgram(program);
			ogsl.setDefaultConstants();
			var cube:CubeTexture = c3d.createCubeTexture(512, Context3DTextureFormat.BGRA, false);
			uploadMip(cube, 0, Bitmap(new CUBE_0()).bitmapData);
			uploadMip(cube, 1, Bitmap(new CUBE_1()).bitmapData);
			uploadMip(cube, 2, Bitmap(new CUBE_2()).bitmapData);
			uploadMip(cube, 3, Bitmap(new CUBE_3()).bitmapData);
			uploadMip(cube, 4, Bitmap(new CUBE_4()).bitmapData);
			uploadMip(cube, 5, Bitmap(new CUBE_5()).bitmapData);
			ogsl.setTexture("bg", cube);
		}

		private function uploadMip(tex:CubeTexture, side:int, bmd:BitmapData):void {
			var size:int = bmd.width;
			var lv:int = 0;
			while (size > 0) {
				var mip:BitmapData = new BitmapData(size, size, false);
				mip.draw(bmd, new Matrix(size / bmd.width, 0, 0, size / bmd.height), null, null, null, true);
				tex.uploadFromBitmapData(mip, side, lv);
				size >>= 1;
				lv++;
			}
		}

		private function loop(e:Event):void {
			mx += (mouseX - mx) * 0.25;
			my += (mouseY - my) * 0.25;
			count++;
			c3d.clear();
			ogsl.setFragmentConstantsFromVector("bgColor", new <Number>[0.1, 0.1, 0.1]);
			// Use a single background color. The original checkerboard pattern resulted
			// meant that a few pixels (along the edge of a sphere) could have completely
			// different colors depending on rounding. This made our image comparison tests
			// much more sensitive to the platform than we would like.
			//ogsl.setFragmentConstantsFromVector("groundColor1", new <Number>[0.4, 0.3, 0.8]);
			ogsl.setFragmentConstantsFromVector("groundColor1", new <Number>[0.8, 0.6, 0.2]);
			ogsl.setFragmentConstantsFromVector("groundColor2", new <Number>[0.8, 0.6, 0.2]);
			ogsl.setFragmentConstantsFromVector("sphereColor1", new <Number>[1, 0.2, 0.2]);
			ogsl.setFragmentConstantsFromVector("sphereColor2", new <Number>[0.2, 1, 0.2]);
			ogsl.setFragmentConstantsFromVector("sphereColor3", new <Number>[0.2, 0.2, 1]);
			var r1:Number = 0.6 + Math.cos(count * 0.04) * 0.1;
			var r2:Number = 0.8 + Math.cos(count * 0.05) * 0.1;
			var r3:Number = 1.2 + Math.cos(count * 0.06) * 0.1;
			ogsl.setFragmentConstantsFromVector("spherePos1", new <Number>[Math.cos(count * 0.05) * 0.5, r1, Math.sin(count * 0.05) * 0.5]);
			ogsl.setFragmentConstantsFromVector("spherePos2", new <Number>[Math.cos(count * 0.04) * 2.0, r2, Math.sin(count * 0.04) * 2.0]);
			ogsl.setFragmentConstantsFromVector("spherePos3", new <Number>[Math.cos(count * 0.03) * 3.5, r3, Math.sin(count * 0.03) * 3.5]);
			ogsl.setFragmentConstantsFromVector("lightPos", new <Number>[Math.cos(count * 0.02) * 2.0, 3, Math.sin(count * 0.02) * 2.0]);
			ogsl.setFragmentConstantsFromNumber("sphereRadius1", r1);
			ogsl.setFragmentConstantsFromNumber("sphereRadius2", r2);
			ogsl.setFragmentConstantsFromNumber("sphereRadius3", r3);
			var hw:Number = WIDTH / 2;
			var hh:Number = HEIGHT / 2;
			ogsl.setFragmentConstantsFromVector("mousePos", new <Number>[(mx - hw) / hw, (hh - my) / hh]);
			ogsl.setFragmentConstantsFromNumber("time", count / 60);
			ogsl.setFragmentConstantsFromNumber("aspect", WIDTH / HEIGHT);
			c3d.drawTriangles(indexBuffer);
			c3d.present();
		}
	}
}

const OGSL_SOURCE:String = <![CDATA[

varying pos:vec2;

program vertex {
	attribute position:vec3;
	uniform matrix:mat4x4;

	function main():void {
		var pos:vec4 = mul(vec4(position, 1), matrix);
		output = pos;
		this.pos = pos.xy;
	}
}

program fragment {
	uniform bgColor:vec3, mousePos:vec2;
	uniform time:float;
	uniform aspect:float; // width / height
	uniform spherePos1:vec3, sphereColor1:vec3, sphereRadius1:float;
	uniform spherePos2:vec3, sphereColor2:vec3, sphereRadius2:float;
	uniform spherePos3:vec3, sphereColor3:vec3, sphereRadius3:float;
	uniform groundColor1:vec3, groundColor2:vec3;
	uniform lightPos:vec3;
	uniform bg:texture;

	function main():void {
		// set screen pos
		var screenPos:vec2 = vec2(pos.x * aspect, pos.y);

		// set camera data
		var cameraPos:vec3 = vec3(cos(mousePos.x * 3.14) * 5, max((mousePos.y + 1) * 4, 0.1), sin(mousePos.x * 3.14) * 5);
		var cameraTarget:vec3 = vec3(0, -1, 0);
		var distanceToScreen:float = 1;
		var cameraY:vec3 = vec3(0, 1, 0);
		var cameraZ:vec3 = normalize(cameraTarget - cameraPos);
		var cameraX:vec3 = normalize(cross(cameraY, cameraZ));
		cameraY = cross(cameraZ, cameraX);

		// set ray data
		var rayPos:vec3 = cameraPos;
		var rayDir:vec3 = normalize(cameraZ * distanceToScreen + cameraX * screenPos.x + cameraY * screenPos.y);
		var t:float, n:vec3, tmin:float, nmin:vec3, cmin:vec3;
		var state:float, goNext:float, hitCount:float, addCubeTex:float;

		goNext = 1;
		hitCount = 0;
		addCubeTex = 0;

		loop(3) {
			if (goNext) {
				state = 0;
				tmin = 100000000;

				t = plane(rayPos, rayDir, vec3(0, 0, 0), n = vec3(0, 1, 0));
				if (t < tmin) {
					tmin = t;
					nmin = n;
					state = 1; // hit to plane
				}

				t = sphere(rayPos, rayDir, spherePos1, sphereRadius1, n);
				if (t < tmin) {
					tmin = t;
					nmin = n;
					cmin = sphereColor1;
					state = 2; // hit to sphere
				}

				t = sphere(rayPos, rayDir, spherePos2, sphereRadius2, n);
				if (t < tmin) {
					tmin = t;
					nmin = n;
					cmin = sphereColor2;
					state = 2; // hit to sphere
				}

				t = sphere(rayPos, rayDir, spherePos3, sphereRadius3, n);
				if (t < tmin) {
					tmin = t;
					nmin = n;
					cmin = sphereColor3;
					state = 2; // hit to sphere
				}

				hitCount += 1;
				if (state == 0) { // did not hit
					addCubeTex = 1;
					goNext = 0;
				} else {
					rayPos += rayDir * tmin;
					rayDir = reflect(rayDir, nmin);
					rayPos += rayDir * 0.0001;
					var toLight:vec3 = lightPos - rayPos;
					var dirToLight:vec3 = normalize(toLight);
					var distanceToLight:float = length(toLight);
					var diffuse:float = saturate(dot(nmin, dirToLight)) / (0.7 + distanceToLight * distanceToLight * 0.01);
					t = sphereDist(rayPos, dirToLight, spherePos1, sphereRadius1);
					if (t < distanceToLight) diffuse = 0;
					t = sphereDist(rayPos, dirToLight, spherePos2, sphereRadius2);
					if (t < distanceToLight) diffuse = 0;
					t = sphereDist(rayPos, dirToLight, spherePos3, sphereRadius3);
					if (t < distanceToLight) diffuse = 0;
					diffuse += saturate(nmin.y) * 0.5;
					var brightness:float = 0.2 + diffuse * 0.6;
					if (state == 1) {
						if (mod(floor(rayPos.x) + floor(rayPos.z), 2) == 1) {
							output.rgb += groundColor1 * brightness;
						} else {
							output.rgb += groundColor2 * brightness;
						}
					}
					if (state == 2) {
						output.rgb += cmin * brightness;
					}
					goNext = 1; // reflect
				}
			}
		}
		var texColor:vec3 = texCube(bg, rayDir, linear).rgb;
		if (addCubeTex) output.rgb += texColor;
		output.rgb /= hitCount;
	}

	function plane(p:vec3, d:vec3, p0:vec3, n:vec3):float {
		var result:float = 100000000;
		var t:float = -dot(p - p0, n) / dot(d, n);
		if (t > 0) result = t;
		return result;
	}

	function sphere(p:vec3, d:vec3, c:vec3, r:float, &n:vec3):float {
		var result:float = 100000000;
		var s:vec3 = p - c;
		var b:float = dot(s, d);
		var det:float = b * b - dot(s, s) + r * r;
		if (det > 0) {
			det = sqrt(det);
			var t:float = -b - det;
			if (t > 0) {
				result = t;
				n = normalize(p + d * t - c);
			}
		}
		return result;
	}

	function sphereDist(p:vec3, d:vec3, c:vec3, r:float):float {
		var result:float = 100000000;
		var s:vec3 = p - c;
		var b:float = dot(s, d);
		var det:float = b * b - dot(s, s) + r * r;
		if (det > 0) {
			det = sqrt(det);
			var t:float = -b - det;
			if (t > 0) {
				result = t;
			}
		}
		return result;
	}
}

]]>

