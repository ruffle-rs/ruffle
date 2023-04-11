package
{
	import com.adobe.utils.*;

	import flash.display.*;
	import flash.display3D.*;
	import flash.display3D.textures.*;
	import flash.events.*;
	import flash.geom.*;
	import flash.text.*;
	import flash.utils.*;
	
	public class SimplePostFilters extends Sprite
	{
		/** Number of degrees to rotate per millisecond */
		private static const ROTATION_SPEED:Number = 1;
		
		/** Axis to rotate about */
		private static const ROTATION_AXIS:Vector3D = new Vector3D(0, 1, 0);
		
		/** UI Padding */
		private static const PAD:Number = 5;
		
		/** Distance between shapes */
		private static const SHAPE_SPACING:Number = 1.5;
		
		[Embed(source="earth.jpg")]
		private static const TEXTURE:Class;
		
		/** Temporary matrix to avoid allocation during drawing */
		private static const TEMP_DRAW_MATRIX:Matrix3D = new Matrix3D();
		
		/** Positions for the corners of the viewport */
		private static const POST_FILTER_POSITIONS:Vector.<Number> = new <Number>[
			-1,  1, // TL
			 1,  1, // TR
			 1, -1, // BR
			-1, -1  // BL
		];
		
		/** Triangles forming a full-viewport quad */
		private static const POST_FILTER_TRIS:Vector.<uint> = new <uint>[
			0, 2, 3, // bottom tri (TL, BR, BL)
			0, 1, 2  // top tri (TL, TR, BR)
		];
		
		/** Constants to pass to the vertex shader for the post filter */
		private static const POST_FILTER_VERTEX_CONSTANTS:Vector.<Number> = new <Number>[1, 2, 0, 0];
		
		/** Constants to pass to the fragment shader for the grayscale post filter */
		private static const GRAYSCALE_FRAGMENT_CONSTANTS:Vector.<Number> = new <Number>[0.3, 0.59, 0.11, 0];
		
		/** Vertex shader for the red-only post filter */
		private var redOnlyProgram:Program3D;
		
		/** Vertex shader for the green-only post filter */
		private var greenOnlyProgram:Program3D;
		
		/** Vertex shader for the blue-only post filter */
		private var blueOnlyProgram:Program3D;
		
		/** Vertex shader for the grayscale post filter */
		private var grayscaleProgram:Program3D;
		
		/** 3D context to draw with */
		private var context3D:Context3D;
		
		/** Shader program to draw with */
		private var program:Program3D;
		
		/** Texture of all shapes */
		private var texture:Texture;
		
		/** Camera viewing the 3D scene */
		private var camera:Camera3D;
		
		/** Shapes to draw */
		private var shapes:Vector.<Shape3D> = new Vector.<Shape3D>();
		
		/** Current rotation of all shapes (degrees) */
		private var rotationDegrees:Number = 0;
		
		/** Number of rows of shapes */
		private var rows:uint = 5;
		
		/** Number of columns of shapes */
		private var cols:uint = 5;
		
		/** Number of layers of shapes */
		private var layers:uint = 1;
		
		/** Framerate display */
		private var fps:TextField = new TextField();
		
		/** Last time the framerate display was updated */
		private var lastFPSUpdateTime:uint;
		
		/** Time when the last frame happened */
		private var lastFrameTime:uint;
		
		/** Number of frames since the framerate display was updated */
		private var frameCount:uint;
		
		/** 3D rendering driver display */
		private var driver:TextField = new TextField();
		
		/** Simulation statistics display */
		private var stats:TextField = new TextField();
		
		/** Name of the filter to use */
		private var filterName:String = "No Filter";
		
		/** Texture the scene is rendered to */
		private var sceneTexture:Texture;
		
		/** Vertex buffer for the full-screen quad to render post-filters with */
		private var postFilterVertexBuffer:VertexBuffer3D;
		
		/** Index buffer for the full-screen quad to render post-filters with */
		private var postFilterIndexBuffer:IndexBuffer3D;
		
		/**
		* Entry point
		*/
		// Ruffle - take in 'Stage' as a parameter
		public function SimplePostFilters(stage:Stage)
		{
			stage.align = StageAlign.TOP_LEFT;
			stage.scaleMode = StageScaleMode.NO_SCALE;
			stage.frameRate = 60;
			
			var stage3D:Stage3D = stage.stage3Ds[0];
			stage3D.addEventListener(Event.CONTEXT3D_CREATE, onContextCreated);
			stage3D.requestContext3D();
		}
		
		protected function onContextCreated(ev:Event): void
		{
			// Setup context
			var stage3D:Stage3D = stage.stage3Ds[0];
			stage3D.removeEventListener(Event.CONTEXT3D_CREATE, onContextCreated);
			context3D = stage3D.context3D;            
			context3D.configureBackBuffer(
				stage.stageWidth,
				stage.stageHeight,
				0,
				true
			);
			context3D.enableErrorChecking = true;
			
			// Setup camera
			camera = new Camera3D(
				0.1, // near
				100, // far
				stage.stageWidth / stage.stageHeight, // aspect ratio
				40*(Math.PI/180), // vFOV
				2, 3, 5, // position
				2, 3, 0, // target
				0, 1, 0 // up dir
			);
			
			// Setup UI
			fps.background = true;
			fps.backgroundColor = 0xffffffff;
			fps.autoSize = TextFieldAutoSize.LEFT;
			fps.text = "Getting FPS...";
			addChild(fps);
			
			driver.background = true;
			driver.backgroundColor = 0xffffffff;
			driver.text = "Driver: " + context3D.driverInfo;
			driver.autoSize = TextFieldAutoSize.LEFT;
			driver.y = fps.height;
			addChild(driver);
			
			stats.background = true;
			stats.backgroundColor = 0xffffffff;
			stats.text = "Getting stats...";
			stats.autoSize = TextFieldAutoSize.LEFT;
			stats.y = driver.y + driver.height;
			addChild(stats);
			
			makeButtons("No Filter", "Red Only", "Green Only", "Blue Only", "Grayscale");
			
			var assembler:AGALMiniAssembler = new AGALMiniAssembler();
			
			// Vertex shader
			var vertSource:String = "m44 op, va0, vc0\nmov v0, va1\n";
			assembler.assemble(Context3DProgramType.VERTEX, vertSource);
			var vertexShaderAGAL:ByteArray = assembler.agalcode;
			
			// Fragment shader
			var fragSource:String = "tex oc, v0, fs0 <2d,linear,mipnone>";
			assembler.assemble(Context3DProgramType.FRAGMENT, fragSource);
			var fragmentShaderAGAL:ByteArray = assembler.agalcode;
			
			// Shader program
			program = context3D.createProgram();
			program.upload(vertexShaderAGAL, fragmentShaderAGAL);
			
			// Setup shapes texture
			var bmd:BitmapData = (new TEXTURE() as Bitmap).bitmapData;
			texture = context3D.createTexture(
				bmd.width,
				bmd.height,
				Context3DTextureFormat.BGRA,
				true
			);
			texture.uploadFromBitmapData(bmd);
			
			// Post filter vertex shader
			vertSource =
				// Pass position through unchanged. It's already in clip space.
				"mov op, va0\n" +
				
				// Position = (position+1)/2
				// Transforms [-1,1] to [0,1]
				"add vt0, vc0.xxxx, va0\n" +
				"div vt0, vt0, vc0.yyyy\n" +
				"sub vt0.y, vc0.x, vt0.y\n" +
				"mov v0, vt0\n";
			assembler.assemble(Context3DProgramType.VERTEX, vertSource);
			vertexShaderAGAL = assembler.agalcode;
			
			// Red-only post filter fragment shader
			fragSource = 
				// Sample scene texture
				"tex ft0, v0, fs0 <2d,clamp,linear>\n" +
				
				// Zero the non-red channels
				"sub ft0.yz, ft0.yz, ft0.yz\n" +
				
				"mov oc, ft0\n";
			assembler.assemble(Context3DProgramType.FRAGMENT, fragSource);
			fragmentShaderAGAL = assembler.agalcode;
			
			// Red-only post filter shader program
			redOnlyProgram = context3D.createProgram();
			redOnlyProgram.upload(vertexShaderAGAL, fragmentShaderAGAL);
			
			// Green-only post filter fragment shader
			fragSource = 
				// Sample scene texture
				"tex ft0, v0, fs0 <2d,clamp,linear>\n" +
				
				// Zero the non-green channels
				"sub ft0.xz, ft0.xz, ft0.xz\n" +
				
				"mov oc, ft0\n";
			assembler.assemble(Context3DProgramType.FRAGMENT, fragSource);
			fragmentShaderAGAL = assembler.agalcode;
			
			// Green-only post filter shader program
			greenOnlyProgram = context3D.createProgram();
			greenOnlyProgram.upload(vertexShaderAGAL, fragmentShaderAGAL);
			
			// Blue-only post filter fragment shader
			fragSource = 
				// Sample scene texture
				"tex ft0, v0, fs0 <2d,clamp,linear>\n" +
				
				// Zero the non-blue channels
				"sub ft0.xy, ft0.xy, ft0.xy\n" +
				
				"mov oc, ft0\n";
			assembler.assemble(Context3DProgramType.FRAGMENT, fragSource);
			fragmentShaderAGAL = assembler.agalcode;
			
			// Blue-only post filter shader program
			blueOnlyProgram = context3D.createProgram();
			blueOnlyProgram.upload(vertexShaderAGAL, fragmentShaderAGAL);
			
			// Grayscale post filter fragment shader
			fragSource = 
				// Sample scene texture
				"tex ft0, v0, fs0 <2d,clamp,linear>\n" +
				
				// Apply coefficients and compute sum
				"dp3 ft0.x, ft0, fc0\n" +
				
				// Copy sum to all channels
				"mov ft0.y, ft0.x\n" +
				"mov ft0.z, ft0.x\n" +
				
				"mov oc, ft0\n";
			assembler.assemble(Context3DProgramType.FRAGMENT, fragSource);
			fragmentShaderAGAL = assembler.agalcode;
			
			// Grayscale post filter shader program
			grayscaleProgram = context3D.createProgram();
			grayscaleProgram.upload(vertexShaderAGAL, fragmentShaderAGAL);
			
			// Setup scene texture
			sceneTexture = context3D.createTexture(
				nextPowerOfTwo(stage.stageWidth),
				nextPowerOfTwo(stage.stageHeight),
				Context3DTextureFormat.BGRA,
				true
			);
			
			// Post filter full-screen quad vertex and index buffers
			postFilterVertexBuffer = context3D.createVertexBuffer(4, 2);
			postFilterVertexBuffer.uploadFromVector(POST_FILTER_POSITIONS, 0, 4);
			postFilterIndexBuffer = context3D.createIndexBuffer(6);
			postFilterIndexBuffer.uploadFromVector(POST_FILTER_TRIS, 0, 6);
			
			makeShapes();
			
			// Start the simulation
			addEventListener(Event.ENTER_FRAME, onEnterFrame);
		}
		
		/**
		*   Get the next-highest power of two
		*   @param v Value to get the next-highest power of two from
		*   @return The next-highest power of two from the given value
		*/
		public static function nextPowerOfTwo(v:uint): uint
		{
			v--;
			v |= v >> 1;
			v |= v >> 2;
			v |= v >> 4;
			v |= v >> 8;
			v |= v >> 16;
			v++;
			return v;
		}
		
		private function makeShapes(): void
		{
			for each (var shape:Shape3D in shapes)
			{
				shape.dispose();
			}
			shapes.length = 0;
			
			for (var row:int = 0; row < rows; ++row)
			{
				for (var col:int = 0; col < cols; ++col)
				{
					for (var layer:int = 0; layer < layers; ++layer)
					{
						var posX:Number = col*SHAPE_SPACING;
						var posY:Number = row*SHAPE_SPACING;
						var posZ:Number = -layer*SHAPE_SPACING;
						
						var rand:Number = Math.random();
						if (rand < 1/6)
						{
							shape = new Cylinder3D(20, context3D, posX, posY, posZ);
						}
						else if (rand < 2/6)
						{
							shape = new Sphere3D(20, 20, context3D, posX, posY, posZ);
						}
						else if (rand < 3/6)
						{
							shape = new Cube3D(context3D, posX, posY, posZ);
						}
						else if (rand < 4/6)
						{
							shape = new Pyramid3D(context3D, posX, posY, posZ);
						}
						else if (rand < 5/6)
						{
							shape = new Circle3D(20, context3D, posX, posY, posZ);
						}
						else
						{
							shape = new Quad3D(context3D, posX, posY, posZ);
						}
						shapes.push(shape);
					}
				}
			}
			
			var numShapes:uint = rows*cols*layers;
			stats.text = "Shapes: (rows=" + rows
				+ ", cols=" + cols
				+ ", layers=" + layers
				+ ", total=" + numShapes + ")";
		}
		
		private function makeButtons(...labels): Number
		{
			var curX:Number = PAD;
			var curY:Number = stage.stageHeight - PAD;
			for each (var label:String in labels)
			{
				if (label == null)
				{
					curX = PAD;
					curY -= button.height + PAD;
					continue;
				}
				
				var tf:TextField = new TextField();
				tf.mouseEnabled = false;
				tf.selectable = false;
				tf.defaultTextFormat = new TextFormat("_sans");
				tf.autoSize = TextFieldAutoSize.LEFT;
				tf.text = label;
				tf.name = "lbl";
				
				var button:Sprite = new Sprite();
				button.buttonMode = true;
				button.graphics.beginFill(0xF5F5F5);
				button.graphics.drawRect(0, 0, tf.width+PAD, tf.height+PAD);
				button.graphics.endFill();
				button.graphics.lineStyle(1);
				button.graphics.drawRect(0, 0, tf.width+PAD, tf.height+PAD);
				button.addChild(tf);
				button.addEventListener(MouseEvent.CLICK, onButton);
				if (curX + button.width > stage.stageWidth - PAD)
				{
					curX = PAD;
					curY -= button.height + PAD;
				}
				button.x = curX;
				button.y = curY - button.height;
				addChild(button);
				
				curX += button.width + PAD;
			}
			
			return curY - button.height;
		}
		
		private function onButton(ev:MouseEvent): void
		{
			filterName = TextField(Sprite(ev.target).getChildByName("lbl")).text;
		}
		
		private function onEnterFrame(ev:Event): void
		{
			switch (filterName)
			{
				case "No Filter":
					renderShapes();
					break;
				case "Red Only":
					renderWithPostFilter(redOnlyProgram, null);
					break;
				case "Green Only":
					context3D.setRenderToBackBuffer();
					renderWithPostFilter(greenOnlyProgram, null);
					break;
				case "Blue Only":
					renderWithPostFilter(blueOnlyProgram, null);
					break;
				case "Grayscale":
					renderWithPostFilter(grayscaleProgram, GRAYSCALE_FRAGMENT_CONSTANTS);
					break;
			}
			context3D.present();
			
			rotationDegrees += ROTATION_SPEED;
			
			// Update stat displays
			frameCount++;
			var now:int = getTimer();
			var elapsed:int = now - lastFPSUpdateTime;
			// Ruffke: Disable this to make the tests deterministic
			if (elapsed > 1000 && false)
			{
				var framerateValue:Number = 1000 / (elapsed / frameCount);
				fps.text = "FPS: " + framerateValue.toFixed(1);
				lastFPSUpdateTime = now;
				frameCount = 0;
			}
			lastFrameTime = now;
		}
		
		private function renderShapes(): void
		{
			// Set up rendering
			context3D.setProgram(program);
			context3D.setTextureAt(0, texture);
			context3D.clear(0.5, 0.5, 0.5);
			
			// Draw shapes
			var worldToClip:Matrix3D = camera.worldToClipMatrix;
			var drawMatrix:Matrix3D = TEMP_DRAW_MATRIX;
			for each (var shape:Shape3D in shapes)
			{
				context3D.setVertexBufferAt(0, shape.positions, 0, Context3DVertexBufferFormat.FLOAT_3);
				context3D.setVertexBufferAt(1, shape.texCoords, 0, Context3DVertexBufferFormat.FLOAT_2);
				
				shape.modelToWorld.copyToMatrix3D(drawMatrix);
				drawMatrix.appendRotation(rotationDegrees, ROTATION_AXIS);
				drawMatrix.prepend(worldToClip);
				context3D.setProgramConstantsFromMatrix(Context3DProgramType.VERTEX, 0, drawMatrix, false);
				context3D.drawTriangles(shape.tris);
			}
		}
		
		private function renderWithPostFilter(program:Program3D, fragConsts:Vector.<Number>): void
		{
			// Render the scene to the scene texture
			context3D.setRenderToTexture(sceneTexture, true, 4);
			renderShapes();
			context3D.setRenderToBackBuffer();
			
			// Render a full-screen quad with the scene texture to the actual screen
			context3D.setProgram(program);
			context3D.setTextureAt(0, sceneTexture);
			context3D.clear(0.5, 0.5, 0.5);
			context3D.setVertexBufferAt(0, postFilterVertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_2);
			context3D.setVertexBufferAt(1, null);
			context3D.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, POST_FILTER_VERTEX_CONSTANTS);
			if (fragConsts)
			{
				context3D.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 0, fragConsts);
			}
			context3D.drawTriangles(postFilterIndexBuffer);
		}
	}
}
