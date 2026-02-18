package {
	import flash.display.Sprite;
	import flash.display.Stage3D;
	import flash.display3D.Context3D;
	import flash.events.Event;
	import flash.geom.Matrix3D;

	public class Test extends Sprite {
		public function Test() {
			stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, onContext3DCreate);
			stage.stage3Ds[0].requestContext3D();
		}

		private function onContext3DCreate(event:Event):void {
			var stage3d:Stage3D = event.target as Stage3D;
			var ctx:Context3D = stage3d.context3D;
			ctx.configureBackBuffer(200, 200, 0, true);

			testSetCulling(ctx);
			testSetProgramConstants(ctx);
			testSetStencilActions(ctx);
			testSetDepthTest(ctx);
			testSetBlendFactors(ctx);
			testSetVertexBufferAt(ctx);
			testSetSamplerStateAt(ctx);
			testCreateTexture(ctx);
			testCreateCubeTexture(ctx);
			testCreateRectangleTexture(ctx);

			trace("Done");
		}

		private function tryCall(label:String, fn:Function):void {
			try {
				fn();
				trace(label + ": OK");
			} catch (e:Error) {
				trace(label + ": " + e);
			}
		}

		// ==========================================
		// setCulling
		// ==========================================
		private function testSetCulling(ctx:Context3D):void {
			trace("=== setCulling ===");

			// -- Valid values --
			tryCall("triangleFace 'none'", function():void {
				ctx.setCulling("none");
			});
			tryCall("triangleFace 'back'", function():void {
				ctx.setCulling("back");
			});
			tryCall("triangleFace 'front'", function():void {
				ctx.setCulling("front");
			});
			tryCall("triangleFace 'frontAndBack'", function():void {
				ctx.setCulling("frontAndBack");
			});

			// -- Invalid values --
			tryCall("triangleFace 'None'", function():void {
				ctx.setCulling("None");
			});
			tryCall("triangleFace 'NONE'", function():void {
				ctx.setCulling("NONE");
			});
			tryCall("triangleFace 'Front'", function():void {
				ctx.setCulling("Front");
			});
			tryCall("triangleFace 'BACK'", function():void {
				ctx.setCulling("BACK");
			});
			tryCall("triangleFace 'FrontAndBack'", function():void {
				ctx.setCulling("FrontAndBack");
			});
			tryCall("triangleFace 'frontandback'", function():void {
				ctx.setCulling("frontandback");
			});
			tryCall("triangleFace 'garbage'", function():void {
				ctx.setCulling("garbage");
			});
		}

		// ==========================================
		// setProgramConstantsFromMatrix / setProgramConstantsFromVector
		// ==========================================
		private function testSetProgramConstants(ctx:Context3D):void {
			trace("=== setProgramConstants ===");

			var mat:Matrix3D = new Matrix3D();
			var vec:Vector.<Number> = new <Number>[0, 0, 0, 0];

			// -- Valid values --
			tryCall("programType 'vertex' (matrix)", function():void {
				ctx.setProgramConstantsFromMatrix("vertex", 0, mat);
			});
			tryCall("programType 'fragment' (matrix)", function():void {
				ctx.setProgramConstantsFromMatrix("fragment", 0, mat);
			});
			tryCall("programType 'vertex' (vector)", function():void {
				ctx.setProgramConstantsFromVector("vertex", 0, vec);
			});
			tryCall("programType 'fragment' (vector)", function():void {
				ctx.setProgramConstantsFromVector("fragment", 0, vec);
			});

			// -- Invalid values --
			tryCall("programType 'Vertex'", function():void {
				ctx.setProgramConstantsFromMatrix("Vertex", 0, mat);
			});
			tryCall("programType 'VERTEX'", function():void {
				ctx.setProgramConstantsFromMatrix("VERTEX", 0, mat);
			});
			tryCall("programType 'Fragment'", function():void {
				ctx.setProgramConstantsFromMatrix("Fragment", 0, mat);
			});
			tryCall("programType 'FRAGMENT'", function():void {
				ctx.setProgramConstantsFromMatrix("FRAGMENT", 0, mat);
			});
			tryCall("programType 'garbage'", function():void {
				ctx.setProgramConstantsFromMatrix("garbage", 0, mat);
			});
			tryCall("programType 'Vertex' (vector)", function():void {
				ctx.setProgramConstantsFromVector("Vertex", 0, vec);
			});
			tryCall("programType 'FRAGMENT' (vector)", function():void {
				ctx.setProgramConstantsFromVector("FRAGMENT", 0, vec);
			});
			tryCall("programType 'garbage' (vector)", function():void {
				ctx.setProgramConstantsFromVector("garbage", 0, vec);
			});
		}

		// ==========================================
		// setStencilActions
		// ==========================================
		private function testSetStencilActions(ctx:Context3D):void {
			trace("=== setStencilActions ===");

			// -- Valid triangleFace values --
			tryCall("triangleFace 'none'", function():void {
				ctx.setStencilActions("none", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'back'", function():void {
				ctx.setStencilActions("back", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'front'", function():void {
				ctx.setStencilActions("front", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'frontAndBack'", function():void {
				ctx.setStencilActions("frontAndBack", "always", "keep", "keep", "keep");
			});

			// -- Invalid triangleFace (case variants) --
			tryCall("triangleFace 'None'", function():void {
				ctx.setStencilActions("None", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'NONE'", function():void {
				ctx.setStencilActions("NONE", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'Front'", function():void {
				ctx.setStencilActions("Front", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'FRONT'", function():void {
				ctx.setStencilActions("FRONT", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'Back'", function():void {
				ctx.setStencilActions("Back", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'BACK'", function():void {
				ctx.setStencilActions("BACK", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'FrontAndBack'", function():void {
				ctx.setStencilActions("FrontAndBack", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'frontandback'", function():void {
				ctx.setStencilActions("frontandback", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'FRONTANDBACK'", function():void {
				ctx.setStencilActions("FRONTANDBACK", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace 'garbage'", function():void {
				ctx.setStencilActions("garbage", "always", "keep", "keep", "keep");
			});
			tryCall("triangleFace ''", function():void {
				ctx.setStencilActions("", "always", "keep", "keep", "keep");
			});

			// -- Valid compareMode values --
			tryCall("compareMode 'never'", function():void {
				ctx.setStencilActions("front", "never", "keep", "keep", "keep");
			});
			tryCall("compareMode 'less'", function():void {
				ctx.setStencilActions("front", "less", "keep", "keep", "keep");
			});
			tryCall("compareMode 'equal'", function():void {
				ctx.setStencilActions("front", "equal", "keep", "keep", "keep");
			});
			tryCall("compareMode 'lessEqual'", function():void {
				ctx.setStencilActions("front", "lessEqual", "keep", "keep", "keep");
			});
			tryCall("compareMode 'greater'", function():void {
				ctx.setStencilActions("front", "greater", "keep", "keep", "keep");
			});
			tryCall("compareMode 'notEqual'", function():void {
				ctx.setStencilActions("front", "notEqual", "keep", "keep", "keep");
			});
			tryCall("compareMode 'greaterEqual'", function():void {
				ctx.setStencilActions("front", "greaterEqual", "keep", "keep", "keep");
			});
			tryCall("compareMode 'always'", function():void {
				ctx.setStencilActions("front", "always", "keep", "keep", "keep");
			});

			// -- Invalid compareMode (case variants) --
			tryCall("compareMode 'Always'", function():void {
				ctx.setStencilActions("front", "Always", "keep", "keep", "keep");
			});
			tryCall("compareMode 'ALWAYS'", function():void {
				ctx.setStencilActions("front", "ALWAYS", "keep", "keep", "keep");
			});
			tryCall("compareMode 'Never'", function():void {
				ctx.setStencilActions("front", "Never", "keep", "keep", "keep");
			});
			tryCall("compareMode 'LessEqual'", function():void {
				ctx.setStencilActions("front", "LessEqual", "keep", "keep", "keep");
			});
			tryCall("compareMode 'lessequal'", function():void {
				ctx.setStencilActions("front", "lessequal", "keep", "keep", "keep");
			});
			tryCall("compareMode 'LESSEQUAL'", function():void {
				ctx.setStencilActions("front", "LESSEQUAL", "keep", "keep", "keep");
			});
			tryCall("compareMode 'GreaterEqual'", function():void {
				ctx.setStencilActions("front", "GreaterEqual", "keep", "keep", "keep");
			});
			tryCall("compareMode 'NotEqual'", function():void {
				ctx.setStencilActions("front", "NotEqual", "keep", "keep", "keep");
			});
			tryCall("compareMode 'garbage'", function():void {
				ctx.setStencilActions("front", "garbage", "keep", "keep", "keep");
			});

			// -- Valid stencil action values (test actionOnBothPass) --
			tryCall("action 'decrementSaturate'", function():void {
				ctx.setStencilActions("front", "always", "decrementSaturate", "keep", "keep");
			});
			tryCall("action 'decrementWrap'", function():void {
				ctx.setStencilActions("front", "always", "decrementWrap", "keep", "keep");
			});
			tryCall("action 'incrementSaturate'", function():void {
				ctx.setStencilActions("front", "always", "incrementSaturate", "keep", "keep");
			});
			tryCall("action 'incrementWrap'", function():void {
				ctx.setStencilActions("front", "always", "incrementWrap", "keep", "keep");
			});
			tryCall("action 'invert'", function():void {
				ctx.setStencilActions("front", "always", "invert", "keep", "keep");
			});
			tryCall("action 'keep'", function():void {
				ctx.setStencilActions("front", "always", "keep", "keep", "keep");
			});
			tryCall("action 'set'", function():void {
				ctx.setStencilActions("front", "always", "set", "keep", "keep");
			});
			tryCall("action 'zero'", function():void {
				ctx.setStencilActions("front", "always", "zero", "keep", "keep");
			});

			// -- Invalid stencil actions (case variants) --
			tryCall("actionOnBothPass 'Keep'", function():void {
				ctx.setStencilActions("front", "always", "Keep", "keep", "keep");
			});
			tryCall("actionOnBothPass 'KEEP'", function():void {
				ctx.setStencilActions("front", "always", "KEEP", "keep", "keep");
			});
			tryCall("actionOnBothPass 'Zero'", function():void {
				ctx.setStencilActions("front", "always", "Zero", "keep", "keep");
			});
			tryCall("actionOnBothPass 'IncrementSaturate'", function():void {
				ctx.setStencilActions("front", "always", "IncrementSaturate", "keep", "keep");
			});
			tryCall("actionOnBothPass 'incrementsaturate'", function():void {
				ctx.setStencilActions("front", "always", "incrementsaturate", "keep", "keep");
			});
			tryCall("actionOnBothPass 'INCREMENTSATURATE'", function():void {
				ctx.setStencilActions("front", "always", "INCREMENTSATURATE", "keep", "keep");
			});
			tryCall("actionOnBothPass 'DecrementWrap'", function():void {
				ctx.setStencilActions("front", "always", "DecrementWrap", "keep", "keep");
			});
			tryCall("actionOnBothPass 'garbage'", function():void {
				ctx.setStencilActions("front", "always", "garbage", "keep", "keep");
			});

			// -- Invalid actionOnDepthFail --
			tryCall("actionOnDepthFail 'Invert'", function():void {
				ctx.setStencilActions("front", "always", "keep", "Invert", "keep");
			});
			tryCall("actionOnDepthFail 'ZERO'", function():void {
				ctx.setStencilActions("front", "always", "keep", "ZERO", "keep");
			});

			// -- Invalid actionOnDepthPassStencilFail --
			tryCall("actionOnDepthPassStencilFail 'Set'", function():void {
				ctx.setStencilActions("front", "always", "keep", "keep", "Set");
			});
			tryCall("actionOnDepthPassStencilFail 'SET'", function():void {
				ctx.setStencilActions("front", "always", "keep", "keep", "SET");
			});
		}

		// ==========================================
		// setDepthTest
		// ==========================================
		private function testSetDepthTest(ctx:Context3D):void {
			trace("=== setDepthTest ===");

			// -- Valid values --
			tryCall("passCompareMode 'never'", function():void {
				ctx.setDepthTest(true, "never");
			});
			tryCall("passCompareMode 'less'", function():void {
				ctx.setDepthTest(true, "less");
			});
			tryCall("passCompareMode 'equal'", function():void {
				ctx.setDepthTest(true, "equal");
			});
			tryCall("passCompareMode 'lessEqual'", function():void {
				ctx.setDepthTest(true, "lessEqual");
			});
			tryCall("passCompareMode 'greater'", function():void {
				ctx.setDepthTest(true, "greater");
			});
			tryCall("passCompareMode 'notEqual'", function():void {
				ctx.setDepthTest(true, "notEqual");
			});
			tryCall("passCompareMode 'greaterEqual'", function():void {
				ctx.setDepthTest(true, "greaterEqual");
			});
			tryCall("passCompareMode 'always'", function():void {
				ctx.setDepthTest(true, "always");
			});

			// -- Invalid values --
			tryCall("passCompareMode 'Less'", function():void {
				ctx.setDepthTest(true, "Less");
			});
			tryCall("passCompareMode 'LESS'", function():void {
				ctx.setDepthTest(true, "LESS");
			});
			tryCall("passCompareMode 'LessEqual'", function():void {
				ctx.setDepthTest(true, "LessEqual");
			});
			tryCall("passCompareMode 'Always'", function():void {
				ctx.setDepthTest(true, "Always");
			});
			tryCall("passCompareMode 'garbage'", function():void {
				ctx.setDepthTest(true, "garbage");
			});
		}

		// ==========================================
		// setBlendFactors
		// ==========================================
		private function testSetBlendFactors(ctx:Context3D):void {
			trace("=== setBlendFactors ===");

			// -- Valid values --
			tryCall("sourceFactor 'one' / destinationFactor 'zero'", function():void {
				ctx.setBlendFactors("one", "zero");
			});
			tryCall("sourceFactor 'sourceAlpha' / destinationFactor 'destinationAlpha'", function():void {
				ctx.setBlendFactors("sourceAlpha", "destinationAlpha");
			});
			tryCall("sourceFactor 'sourceColor' / destinationFactor 'destinationColor'", function():void {
				ctx.setBlendFactors("sourceColor", "destinationColor");
			});
			tryCall("sourceFactor 'oneMinusSourceAlpha' / destinationFactor 'oneMinusDestinationAlpha'", function():void {
				ctx.setBlendFactors("oneMinusSourceAlpha", "oneMinusDestinationAlpha");
			});
			tryCall("sourceFactor 'oneMinusSourceColor' / destinationFactor 'oneMinusDestinationColor'", function():void {
				ctx.setBlendFactors("oneMinusSourceColor", "oneMinusDestinationColor");
			});

			// -- Invalid sourceFactor --
			tryCall("sourceFactor 'One'", function():void {
				ctx.setBlendFactors("One", "zero");
			});
			tryCall("sourceFactor 'ONE'", function():void {
				ctx.setBlendFactors("ONE", "zero");
			});
			tryCall("sourceFactor 'SourceAlpha'", function():void {
				ctx.setBlendFactors("SourceAlpha", "zero");
			});
			tryCall("sourceFactor 'sourcealpha'", function():void {
				ctx.setBlendFactors("sourcealpha", "zero");
			});
			tryCall("sourceFactor 'SOURCEALPHA'", function():void {
				ctx.setBlendFactors("SOURCEALPHA", "zero");
			});
			tryCall("sourceFactor 'OneMinusSourceAlpha'", function():void {
				ctx.setBlendFactors("OneMinusSourceAlpha", "zero");
			});
			tryCall("sourceFactor 'garbage'", function():void {
				ctx.setBlendFactors("garbage", "zero");
			});

			// -- Invalid destinationFactor --
			tryCall("destinationFactor 'Zero'", function():void {
				ctx.setBlendFactors("one", "Zero");
			});
			tryCall("destinationFactor 'ZERO'", function():void {
				ctx.setBlendFactors("one", "ZERO");
			});
			tryCall("destinationFactor 'DestinationAlpha'", function():void {
				ctx.setBlendFactors("one", "DestinationAlpha");
			});
			tryCall("destinationFactor 'garbage'", function():void {
				ctx.setBlendFactors("one", "garbage");
			});
		}

		// ==========================================
		// setVertexBufferAt
		// ==========================================
		private function testSetVertexBufferAt(ctx:Context3D):void {
			trace("=== setVertexBufferAt ===");

			var vb = ctx.createVertexBuffer(3, 4);

			// -- Valid values --
			tryCall("format 'float1'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "float1");
			});
			tryCall("format 'float2'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "float2");
			});
			tryCall("format 'float3'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "float3");
			});
			tryCall("format 'float4'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "float4");
			});
			tryCall("format 'bytes4'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "bytes4");
			});

			// -- Invalid values --
			tryCall("format 'Float1'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "Float1");
			});
			tryCall("format 'FLOAT1'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "FLOAT1");
			});
			tryCall("format 'Bytes4'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "Bytes4");
			});
			tryCall("format 'BYTES4'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "BYTES4");
			});
			tryCall("format 'garbage'", function():void {
				ctx.setVertexBufferAt(0, vb, 0, "garbage");
			});

			// Clean up
			ctx.setVertexBufferAt(0, null, 0, "float4");
		}

		// ==========================================
		// setSamplerStateAt
		// ==========================================
		private function testSetSamplerStateAt(ctx:Context3D):void {
			trace("=== setSamplerStateAt ===");

			// -- Valid wrap values --
			tryCall("wrap 'clamp'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "linear", "mipnone");
			});
			tryCall("wrap 'repeat'", function():void {
				ctx.setSamplerStateAt(0, "repeat", "linear", "mipnone");
			});
			tryCall("wrap 'clamp_u_repeat_v'", function():void {
				ctx.setSamplerStateAt(0, "clamp_u_repeat_v", "linear", "mipnone");
			});
			tryCall("wrap 'repeat_u_clamp_v'", function():void {
				ctx.setSamplerStateAt(0, "repeat_u_clamp_v", "linear", "mipnone");
			});

			// -- Invalid wrap --
			tryCall("wrap 'Clamp'", function():void {
				ctx.setSamplerStateAt(0, "Clamp", "linear", "mipnone");
			});
			tryCall("wrap 'CLAMP'", function():void {
				ctx.setSamplerStateAt(0, "CLAMP", "linear", "mipnone");
			});
			tryCall("wrap 'Repeat'", function():void {
				ctx.setSamplerStateAt(0, "Repeat", "linear", "mipnone");
			});
			tryCall("wrap 'clampURepeatV'", function():void {
				ctx.setSamplerStateAt(0, "clampURepeatV", "linear", "mipnone");
			});
			tryCall("wrap 'garbage'", function():void {
				ctx.setSamplerStateAt(0, "garbage", "linear", "mipnone");
			});

			// -- Valid filter values --
			tryCall("filter 'linear'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "linear", "mipnone");
			});
			tryCall("filter 'nearest'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "nearest", "mipnone");
			});

			// -- Invalid filter --
			tryCall("filter 'Linear'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "Linear", "mipnone");
			});
			tryCall("filter 'LINEAR'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "LINEAR", "mipnone");
			});
			tryCall("filter 'Nearest'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "Nearest", "mipnone");
			});
			tryCall("filter 'garbage'", function():void {
				ctx.setSamplerStateAt(0, "clamp", "garbage", "mipnone");
			});
		}

		// ==========================================
		// createTexture
		// ==========================================
		private function testCreateTexture(ctx:Context3D):void {
			trace("=== createTexture ===");

			// -- Valid values --
			tryCall("textureFormat 'bgra'", function():void {
				ctx.createTexture(64, 64, "bgra", false, 0);
			});
			tryCall("textureFormat 'compressed'", function():void {
				ctx.createTexture(64, 64, "compressed", false, 0);
			});
			tryCall("textureFormat 'compressedAlpha'", function():void {
				ctx.createTexture(64, 64, "compressedAlpha", false, 0);
			});

			// -- Invalid values --
			tryCall("textureFormat 'Bgra'", function():void {
				ctx.createTexture(64, 64, "Bgra", false, 0);
			});
			tryCall("textureFormat 'BGRA'", function():void {
				ctx.createTexture(64, 64, "BGRA", false, 0);
			});
			tryCall("textureFormat 'Compressed'", function():void {
				ctx.createTexture(64, 64, "Compressed", false, 0);
			});
			tryCall("textureFormat 'CompressedAlpha'", function():void {
				ctx.createTexture(64, 64, "CompressedAlpha", false, 0);
			});
			tryCall("textureFormat 'compressedalpha'", function():void {
				ctx.createTexture(64, 64, "compressedalpha", false, 0);
			});
			tryCall("textureFormat 'garbage'", function():void {
				ctx.createTexture(64, 64, "garbage", false, 0);
			});
		}

		// ==========================================
		// createCubeTexture
		// ==========================================
		private function testCreateCubeTexture(ctx:Context3D):void {
			trace("=== createCubeTexture ===");

			// -- Valid values --
			tryCall("textureFormat 'bgra'", function():void {
				ctx.createCubeTexture(64, "bgra", false, 0);
			});
			tryCall("textureFormat 'compressed'", function():void {
				ctx.createCubeTexture(64, "compressed", false, 0);
			});

			// -- Invalid values --
			tryCall("textureFormat 'Bgra'", function():void {
				ctx.createCubeTexture(64, "Bgra", false, 0);
			});
			tryCall("textureFormat 'COMPRESSED'", function():void {
				ctx.createCubeTexture(64, "COMPRESSED", false, 0);
			});
			tryCall("textureFormat 'garbage'", function():void {
				ctx.createCubeTexture(64, "garbage", false, 0);
			});
		}

		// ==========================================
		// createRectangleTexture
		// ==========================================
		private function testCreateRectangleTexture(ctx:Context3D):void {
			trace("=== createRectangleTexture ===");

			// -- Valid values --
			tryCall("textureFormat 'bgra'", function():void {
				ctx.createRectangleTexture(64, 64, "bgra", false);
			});
			// Note: "compressed" throws Error #3762 on rectangle textures (not supported),
			// testing that is out of scope for string arg validation.

			// -- Invalid values --
			tryCall("textureFormat 'Bgra'", function():void {
				ctx.createRectangleTexture(64, 64, "Bgra", false);
			});
			tryCall("textureFormat 'BGRA'", function():void {
				ctx.createRectangleTexture(64, 64, "BGRA", false);
			});
			tryCall("textureFormat 'garbage'", function():void {
				ctx.createRectangleTexture(64, 64, "garbage", false);
			});
		}
	}
}
