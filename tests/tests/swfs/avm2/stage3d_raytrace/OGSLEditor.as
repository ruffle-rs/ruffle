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
	import com.bit101.components.*;
	import com.element.oimo.ogsl.OGSL;
	import flash.display.*;
	import flash.display3D.*;
	import flash.display3D.textures.CubeTexture;
	import flash.events.*;
	import flash.geom.*;
	import flash.text.TextField;
	import flash.ui.Keyboard;

	/**
	 * OGSL real-time editor
	 *   * MinimalComps library is required
	 * @author saharan
	 */
	[SWF(width = "900", height = "600", frameRate = "60")]
	public class OGSLEditor extends Sprite {
		private const WIDTH:int = 400;
		private const HEIGHT:int = 400;
		private var s3d:Stage3D;
		private var c3d:Context3D;
		private var vbuf:VertexBuffer3D;
		private var ibuf:IndexBuffer3D;
		private var ogsl:OGSL;
		private var count:int;

		public function OGSLEditor() {
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

			createVertexBuffer();
			createIndexBuffer();

			initGUI();

			addEventListener(Event.ENTER_FRAME, loop);
		}

		private function initGUI():void {
			Style.embedFonts = false;
			Style.fontName = "Courier New";
			Style.fontSize = 12;
			var ta:TextArea = new TextArea(stage, 400, 0, "// --- OGSL source code ---\r\rvarying pos:vec2; // position on screen\r\rprogram vertex {\r    attribute position:vec3;\r    uniform matrix:mat4x4;\r\r    function main():void {\r        output = mul(matrix, vec4(position, 1));\r        pos = position.xy;\r    }\r}\r\rprogram fragment {\r    uniform mousePos:vec2;\r    uniform time:float;\r\r    function main():void {\r        \r        // --- try editing here! ---\r        \r        var dist:float = distance(mousePos, pos);\r        dist += (sin(pos.x * 32) + sin(pos.y * 32)) * 0.06;\r        var brightness:float = max(1 - dist, 0);\r        var theta:float = dist * 3 + time;\r        var color:vec3;\r        color.r = sin(theta * 2.3);\r        color.g = sin(theta * 2.5);\r        color.b = sin(theta * 2.7);\r        color = color * 0.5 + 0.5;\r        output.rgb = color * round(brightness * 16) / 16;\r    }\r}");
			var caretShiftCount:int = 0;
			ta.addEventListener(KeyboardEvent.KEY_DOWN, function(e:KeyboardEvent):void {
				var tf:TextField = ta.textField;
				var text:String = tf.text;
				var beginIndex:int = tf.selectionBeginIndex;
				var endIndex:int = tf.selectionEndIndex;
				if (e.keyCode == Keyboard.TAB) {
					tf.text = text.substring(0, beginIndex) + "    " + text.substring(endIndex);
					tf.setSelection(beginIndex + 4, beginIndex + 4);
				}
				if (e.keyCode == Keyboard.ENTER) {
					var lastLine:int = text.lastIndexOf("\r", beginIndex - 1) + 1;
					if (lastLine != 0) {
						var numSpaces:int = 0;
						var spaces:String = "";
						while (text.charAt(lastLine + numSpaces) == " ") {
							numSpaces++;
							spaces += " ";
							tf.text = text.substring(0, beginIndex) + spaces + text.substring(endIndex);
							caretShiftCount = numSpaces;
						}
					}
				}
			});
			ta.textField.addEventListener(Event.CHANGE, function(e:Event):void {
				var tf:TextField = ta.textField;
				if (caretShiftCount != 0) {
					tf.setSelection(tf.selectionBeginIndex + caretShiftCount, tf.selectionEndIndex + caretShiftCount);
					caretShiftCount = 0;
				}
			});
			ta.editable = true;
			ta.width = 500;
			ta.height = 560;
			var compile:PushButton = new PushButton(stage, 400, 560, "Compile!", function(e:Event):void {
				log.text = "--- Compiler log ---\n\n";
				try {
					ogsl.compile(ta.text);
				} catch (err:Error) {
					log.text += "Error! message: " + err.message + "\n";
				}
				if (ogsl.isCompiled()) {
					log.text += "Compiling finished successfully.\n\nvertex AGAL:\n" + ogsl.getVertexAGAL() + "\nfragment AGAL:\n" + ogsl.getFragmentAGAL();
					var vc:Vector.<Number> = ogsl.getDefaultVertexConstantsData();
					var fc:Vector.<Number> = ogsl.getDefaultFragmentConstantsData();
					log.text += "\ndefault vertex constants:\n";
					for (var i:int = 0; i < vc.length; i += 5) {
						log.text += "vc" + vc[i] + " = (" + vc[i + 1] + ", " + vc[i + 2] + ", " + vc[i + 3] + ", " + vc[i + 4] + ")\n";
					}
					log.text += "\ndefault fragment constants:\n";
					for (i = 0; i < fc.length; i += 5) {
						log.text += "fc" + fc[i] + " = (" + fc[i + 1] + ", " + fc[i + 2] + ", " + fc[i + 3] + ", " + fc[i + 4] + ")\n";
					}
					ogsl.setContext3D(c3d);
					var assembler:AGALMiniAssembler = new AGALMiniAssembler();
					var program:Program3D = c3d.createProgram();
					program.upload(
						assembler.assemble(Context3DProgramType.VERTEX, ogsl.getVertexAGAL(), 2),
						assembler.assemble(Context3DProgramType.FRAGMENT, ogsl.getFragmentAGAL(), 2)
					);
					c3d.setProgram(program);
					count = 0;
					setOGSLConstants();
				}
			});
			compile.width = 500;
			compile.height = 40;
			var log:TextArea = new TextArea(stage, 0, 400, "log");
			log.editable = false;
			log.width = 400;
			log.height = 200;
			stage.addEventListener(FocusEvent.KEY_FOCUS_CHANGE, function(e:Event):void { e.preventDefault(); });
			compile.dispatchEvent(new MouseEvent(MouseEvent.CLICK));
		}

		private function setOGSLConstants():void {
			ogsl.setDefaultConstants();
			ogsl.setVertexConstantsFromMatrix("matrix", new Matrix3D());
			ogsl.setVertexBuffer("position", vbuf, 0, Context3DVertexBufferFormat.FLOAT_3);
		}

		private function createVertexBuffer():void {
			vbuf = c3d.createVertexBuffer(4, 3);
			vbuf.uploadFromVector(Vector.<Number>([
				-1, -1, 0, // x, y, z
				 1, -1, 0,
				-1,  1, 0,
				 1,  1, 0,
			]), 0, 4);
		}

		private function createIndexBuffer():void {
			ibuf = c3d.createIndexBuffer(6);
			ibuf.uploadFromVector(Vector.<uint>([
				0, 1, 2,
				1, 3, 2,
			]), 0, 6);
		}

		private function loop(e:Event):void {
			count++;
			if (ogsl.isCompiled()) {
				c3d.clear();
				var halfWidth:Number = WIDTH / 2;
				var halfHeight:Number = HEIGHT / 2;
				ogsl.setFragmentConstantsFromVector("mousePos", new <Number>[(mouseX - halfWidth) / halfWidth, (halfHeight - mouseY) / halfHeight]);
				ogsl.setFragmentConstantsFromNumber("time", count / 60);
				c3d.drawTriangles(ibuf);
				c3d.present();
			} else {
				c3d.clear(0.3, 0.3, 0.3);
				c3d.present();
			}
		}
	}
}
