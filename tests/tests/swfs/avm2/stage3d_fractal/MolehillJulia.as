/* MolehillJula
 * mwelsh@gmail.com
 *
 * Copyright (C) 2011 by Michael R. Welsh

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THIS SOFTWARE
*/

package
{
	import com.adobe.utils.AGALMiniAssembler;
	import flash.display.Sprite;
	import flash.display.Stage3D;
	import flash.display.StageAlign;
	import flash.display.StageScaleMode;
	import flash.display3D.Context3D;
	import flash.display3D.Context3DProgramType;
	import flash.display3D.Context3DRenderMode;
	import flash.display3D.Context3DVertexBufferFormat;
	import flash.display3D.IndexBuffer3D;
	import flash.display3D.Program3D;
	import flash.display3D.VertexBuffer3D;
	import flash.events.ContextMenuEvent;
	import flash.events.Event;
	import flash.events.KeyboardEvent;
	import flash.events.MouseEvent;
	import flash.events.TimerEvent;
	import flash.filters.DropShadowFilter;
	import flash.geom.Matrix3D;
	import flash.geom.Rectangle;
	import flash.geom.Vector3D;
	import flash.net.navigateToURL;
	import flash.net.URLRequest;
	import flash.text.TextField;
	import flash.text.TextFieldAutoSize;
	import flash.text.TextFormat;
	import flash.ui.ContextMenu;
	import flash.ui.ContextMenuItem;
	import flash.ui.Keyboard;
	import flash.ui.Mouse;
	import flash.ui.MouseCursor;
	import flash.utils.ByteArray;
	import flash.utils.getTimer;
	import flash.utils.Timer;

	/**
	 * ...
	 * @author Mike Welsh
	 */
	[SWF(width='800', height='600', backgroundColor='#000000', frameRate='65')]
	public class MolehillJulia extends Sprite
	{
		
		public function MolehillJulia():void
		{
			if (stage) init();
			else addEventListener(Event.ADDED_TO_STAGE, init);
		}

		private static const VERT_SIZE:uint					= 4;
		private static const NUM_ITERATIONS:uint			= 23;
		private static const Z_AXIS:Vector3D				= new Vector3D(0, 0, 1.0);
		
		private static const ROTATION_SPEED:Number			= 1.0;
		private static const SCROLL_SPEED:Number			= 0.01;
		private static const SCALE_SPEED:Number				= 0.02;
		private static const C_SPEED:Number					= 0.01;
		private static const MOUSE_SENSITIVITY:Number		= 0.1;
		private static const MOUSE_WHEEL_SENSITIVITY:Number	= 0.5;
		private static const ANIMATION_SPEED:Number			= 1/2000;
		private static const SLIDE_FACTOR:Number			= 0.3;
		
		private var _context:Context3D;
		private var _vertexBuffer:VertexBuffer3D;
		private var _indexBuffer:IndexBuffer3D;
		private var _transform:Matrix3D;
		
		private var _animating:Boolean;
		
		private var _x:Number			= 0.0;
		private var _y:Number			= 0.0;
		private var _targetX:Number		= 0.0;
		private var _targetY:Number		= 0.0;
		private var _rotation:Number	= 0.0;
		private var _targetScale:Number	= 1.0;
		private var _scale:Number		= 1.0;
		
		private var _mouseX:Number;
		private var _mouseY:Number;
		private var _mousePos:Vector3D;
		
		private var _colorSine:Number;
		private var _colorShift:Vector.<Number>;
		private var _c:Vector.<Number>;
		
		private var _keys:Vector.<Boolean>;
		
		private var _posText:TextField;
		private var _infoText:TextField;
		
		private var _resizeTimer:Timer;
		
		private function init(e:Event = null):void
		{
			stage.scaleMode = StageScaleMode.NO_SCALE;
			stage.align = StageAlign.TOP_LEFT
			
			_c = Vector.<Number>([0.25, -0.575, 0, 0]);
			_colorShift = Vector.<Number>([0, 0, 0, 1.0]);
			_keys = new Vector.<Boolean>(256, true);
			
			_mousePos = new Vector3D();
			
			_colorSine = 10.0;
			updateColors();
			
			stage.addEventListener(Event.RESIZE, onResize);
			stage.addEventListener(KeyboardEvent.KEY_DOWN, onKeyChanged);
			stage.addEventListener(KeyboardEvent.KEY_UP, onKeyChanged);
			stage.addEventListener(MouseEvent.MOUSE_WHEEL, onMouseWheel);
			stage.addEventListener(MouseEvent.DOUBLE_CLICK, onDoubleClick);
			stage.addEventListener(MouseEvent.MOUSE_MOVE, onMouseMove);
			
			stage.doubleClickEnabled = true;
			
			Mouse.cursor = MouseCursor.HAND;
			stage.showDefaultContextMenu = false;
			contextMenu = new ContextMenu();
			var cMenuItem:ContextMenuItem = new ContextMenuItem("MolehillJulia by Mike Welsh");
			cMenuItem.addEventListener(ContextMenuEvent.MENU_ITEM_SELECT,
				function(event:Event):void { navigateToURL(new URLRequest("http://www.gingerbinger.com")); }
			);
			contextMenu.customItems.push(cMenuItem);
			
			setupUI();
			
			_resizeTimer = new Timer(250, 1);
			_resizeTimer.addEventListener(TimerEvent.TIMER, onResizeTimer);
			
			createContext();
		}
		
		private function createContext():void
		{
			if (_context)
			{
				stage.removeEventListener(Event.ENTER_FRAME, onEnterFrame);
				_context.dispose();
				_context = null;
			}
			
			var stage3D:Stage3D = stage.stage3Ds[0];
			stage3D.x = stage3D.y = 0;
			stage3D.addEventListener(Event.CONTEXT3D_CREATE, onContextCreated);
			stage3D.requestContext3D(Context3DRenderMode.AUTO);
		}
		
		private function onContextCreated(event:Event):void
		{
			var stage3D:Stage3D = event.target as Stage3D;
			_context = stage3D.context3D;
			
			if (!_context) return;
			_context.enableErrorChecking = true;
			_context.configureBackBuffer(stage.stageWidth, stage.stageHeight, 0);
			
			var vertexShader:AGALMiniAssembler = new AGALMiniAssembler();
			vertexShader.assemble(Context3DProgramType.VERTEX,
				"mov op, va0				\n" +
				"m44 v0, va1, vc1			\n" +
				"mov v0.zw, vc0.zw"
			);
			
			var fragmentShader:AGALMiniAssembler = new AGALMiniAssembler();
			var fragmentShaderCode:String =
				"mov ft0.xy, v0.xy				\n" +
				"mov ft0.zw, fc0.xx				\n" +
				"mov ft1.x, fc0.x				\n";

			for (var i:uint = 0; i < NUM_ITERATIONS; i++)
				fragmentShaderCode +=
				"neg ft0.z, ft0.y				\n" +
				"mul ft3.x, fc0.z, ft0.x 		\n" +
				"dp3 ft0.x, ft0.xyw, ft0.xzw	\n" +
				"mul ft0.y, ft3.x, ft0.y		\n" +
				"add ft0.xy, ft0.xy, fc3.xy		\n" +
				"dp3 ft2.x, ft0.xyw, ft0.xyw	\n" +
				"slt ft3.x, ft2.x, fc0.w		\n" +
				"add ft1.x, ft1.x, ft3.x		\n";
				
			fragmentShaderCode +=
				"div ft2.x, ft1.x, fc2.y		\n" +
				"add ft2, ft2.xxxx, fc1			\n" +
				"add ft2, ft2, fc1.wwww			\n" +
				"div ft2, ft2, fc0.zzzz			\n" +
				"frc ft2, ft2					\n" +
				"mul ft2, ft2, fc0.zzzz			\n" +
				"sub ft2, ft2, fc2.zzzz			\n" +
				"abs ft2, ft2					\n" +
				"slt ft1, ft1.xxxx, fc2.yyyy	\n" +
				"mul oc, ft1, ft2				";
				
			fragmentShader.assemble(Context3DProgramType.FRAGMENT, fragmentShaderCode);
			
			var program:Program3D = _context.createProgram();
			program.upload(vertexShader.agalcode, fragmentShader.agalcode);
			_context.setProgram(program);
			
			var scale:Number = (stage.stageWidth / stage.stageHeight);
			var verts:Vector.<Number> = Vector.<Number>(
				[
					-1.0,	1.0,	-scale,	1.0,
					1.0,	1.0,	scale,	1.0,
					1.0,	-1.0,	scale,	-1.0,
					-1.0,	-1.0,	-scale,	-1.0
				]);
			_vertexBuffer = _context.createVertexBuffer(verts.length / VERT_SIZE, VERT_SIZE);
			_vertexBuffer.uploadFromVector(verts, 0, verts.length / VERT_SIZE);

			var indices:Vector.<uint> = Vector.<uint>([0, 1, 2, 0, 2, 3]);
			_indexBuffer = _context.createIndexBuffer(indices.length);
			_indexBuffer.uploadFromVector(indices, 0, indices.length);
			
			_context.setVertexBufferAt(0, _vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_2);
			_context.setVertexBufferAt(1, _vertexBuffer, 2, Context3DVertexBufferFormat.FLOAT_2);
			
			_context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, Vector.<Number>([0, 0, 0, 1]));
			_context.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 0, Vector.<Number>([0, 0, 2, 4]));
			_context.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 1, Vector.<Number>(_colorShift));
			_context.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 2, Vector.<Number>([10, NUM_ITERATIONS, 1.0, -1.0]));
			
			_context.setColorMask(true, true, true, false);
			
			_infoText.y = stage.stageHeight - 36;
			
			_transform = new Matrix3D();
			
			stage.addEventListener(Event.ENTER_FRAME, onEnterFrame);
		}
		
		private function setupUI():void
		{
			var shadow:DropShadowFilter = new DropShadowFilter();
			var textFormat:TextFormat = new TextFormat("_sans", 12, 0xffffff, true);
			
			_posText = new TextField();
			_posText.selectable = false;
			_posText.alpha = 0.7;
			_posText.filters = [shadow];
			_posText.y = 0;
			_posText.autoSize = TextFieldAutoSize.LEFT;
			_posText.defaultTextFormat = textFormat;
			addChild(_posText);
			
			_infoText = new TextField();
			_infoText.selectable = false;
			_infoText.alpha = 0.7;
			_infoText.filters = [shadow];
			_infoText.y = stage.stageHeight - 36;
			_infoText.autoSize = TextFieldAutoSize.LEFT;
			_infoText.defaultTextFormat = textFormat;
			_infoText.text = "ARROWS/MOUSE to scroll.\tMOUSE WHEEL/+- to scale.\tSHIFT+ARROWS/MOUSE to morph.\nSPACE for animation.\t\tCTRL+ARROWS/MOUSE to color cycle.";
			addChild(_infoText);
		}
		
		private function onEnterFrame(event:Event):void
		{
			if (!_context) return;
			
			if (_keys[Keyboard.SHIFT])
			{
				if (_keys[Keyboard.LEFT])		_c[0] -= C_SPEED;
				if (_keys[Keyboard.RIGHT])		_c[0] += C_SPEED;
				if (_keys[Keyboard.UP])			_c[1] -= C_SPEED;
				if (_keys[Keyboard.DOWN])		_c[1] += C_SPEED;
				if (_keys[Keyboard.NUMPAD_ADD] || _keys[Keyboard.EQUAL])
					_rotation -= ROTATION_SPEED;
				if (_keys[Keyboard.NUMPAD_SUBTRACT] || _keys[Keyboard.MINUS])
					_rotation += ROTATION_SPEED;
			}
			else if (_keys[Keyboard.CONTROL])
			{
				var dc:Number = 0;
				if (_keys[Keyboard.LEFT])	_colorShift[3] -= C_SPEED;
				if (_keys[Keyboard.RIGHT]) 	_colorShift[3] += C_SPEED;
				if (_keys[Keyboard.UP]) 	dc -= C_SPEED;
				if (_keys[Keyboard.DOWN])	dc += C_SPEED;
				if (dc)
				{
					_colorSine += dc;
					updateColors();
				}
			}
			else
			{
				var angle:Number = _rotation*Math.PI/180;
				var cos:Number = Math.cos(angle);
				var sin:Number = Math.sin(angle);
				var dx:Number = 0, dy:Number = 0;
				if (_keys[Keyboard.LEFT])		dx -= SCROLL_SPEED;
				if (_keys[Keyboard.RIGHT])		dx += SCROLL_SPEED;
				if (_keys[Keyboard.UP])			dy += SCROLL_SPEED;
				if (_keys[Keyboard.DOWN])		dy -= SCROLL_SPEED;
				if(dx || dy) scroll(dx, dy);
				if (_keys[Keyboard.NUMPAD_ADD] || _keys[Keyboard.EQUAL])
					_targetScale *= 1.0+SCALE_SPEED;
				if (_keys[Keyboard.NUMPAD_SUBTRACT] || _keys[Keyboard.MINUS])
					_targetScale *= 1.0-SCALE_SPEED;
			}
			
			if (_animating)
			{
				var t:Number = getTimer() * ANIMATION_SPEED;
				_infoText.alpha -= 0.1;
				_c[0] = Math.sin(t*0.9);
				_c[1] = Math.sin(t*1.23);
				_colorShift[3] = t;
				_colorSine = t * 0.2;
				updateColors();
			}

			_targetX = Math.max(-1, Math.min(1, _targetX));
			_targetY = Math.max( -1, Math.min(1, _targetY));
						
			_c[0] = Math.max( -2, Math.min(2, _c[0]));
			_c[1] = Math.max( -2, Math.min(2, _c[1]));
			_targetScale = Math.min(50000, Math.max(0.25, _targetScale) );

			_x += (_targetX - _x) * SLIDE_FACTOR;
			_y += (_targetY - _y) * SLIDE_FACTOR;
			_scale += (_targetScale - _scale) * SLIDE_FACTOR;
			if (Math.abs(_targetScale-_scale) < 0.0001) _scale = _targetScale;
			
			_transform.identity();
			_transform.appendScale(1/_scale, 1/_scale, 1);
			_transform.appendRotation(_rotation, Z_AXIS);
			_transform.appendTranslation(_x, _y, 0);
		
			_mousePos.x = (stage.mouseX - stage.stageWidth/2)/(stage.stageHeight/2);
			_mousePos.y = (stage.stageHeight/2 - stage.mouseY)/(stage.stageHeight/2);
			_mousePos = _transform.transformVector(_mousePos);
			_posText.text = "pos =\t(" + _mousePos.x + ",\t" + _mousePos.y + ")\n";
			_posText.appendText("c =\t\t(" + _c[0] + ",\t" + _c[1] + ")\n");
			_posText.appendText("zoom = \t" + _scale);
			
			_context.clear();
			_context.setProgramConstantsFromMatrix(Context3DProgramType.VERTEX, 1, _transform, true);
			_context.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 1, _colorShift);
			_context.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 3, _c);
			_context.drawTriangles(_indexBuffer);
			_context.present();
		}
		
		private function onKeyChanged(event:KeyboardEvent):void
		{
			_keys[event.keyCode] = event.type == KeyboardEvent.KEY_DOWN;
			
			if (_keys[Keyboard.SPACE]) _animating = !_animating;
			if (_keys[Keyboard.ESCAPE])
			{
				_targetX = _targetY = 0;
				_targetScale = 1;
			}
		}
		
		private function scroll(dx:Number, dy:Number):void
		{
			dx /= _scale;
			dy /= _scale;
			var angle:Number = _rotation*Math.PI/180;
			var cos:Number = Math.cos(angle);
			var sin:Number = Math.sin(angle);
			_targetX += dx*cos - dy*sin;
			_targetY += dx*sin + dy*cos;
		}
		
		private function onMouseMove(event:MouseEvent):void
		{
			var dx:Number = stage.mouseX - _mouseX;
			var dy:Number = stage.mouseY - _mouseY;
			dx *= MOUSE_SENSITIVITY;
			dy *= MOUSE_SENSITIVITY;
			_mouseX = stage.mouseX;
			_mouseY = stage.mouseY;
			
			if (!event.buttonDown) return;
			
			if (_keys[Keyboard.SHIFT])
			{
				_c[0] += dx * C_SPEED;
				_c[1] += dy * C_SPEED;
			}
			else if (_keys[Keyboard.CONTROL])
			{
				if (dy)
				{
					_colorSine += dy * C_SPEED;
					updateColors();
				}
				_colorShift[3] += dx * C_SPEED;
			}
			else
			{
				scroll(-dx * SCROLL_SPEED, dy * SCROLL_SPEED);
			}
		}
		
		private function onMouseWheel(event:MouseEvent):void
		{
			if (_keys[Keyboard.SHIFT])
				_rotation -= event.delta * MOUSE_WHEEL_SENSITIVITY * ROTATION_SPEED;
			else
				_targetScale *= 1.0 + event.delta * MOUSE_WHEEL_SENSITIVITY * SCALE_SPEED;
		}
		
		private function onResize(event:Event):void
		{
			if (_resizeTimer)
			{
				_resizeTimer.reset();
				_resizeTimer.start();
			}
		}

		private function onResizeTimer(event:Event):void
		{
			createContext();
		}
		
		private function updateColors():void
		{
			_colorShift[0] = Math.sin( _colorSine*Math.sqrt(5) ) * 0.5 + 0.5;
			_colorShift[1] = Math.sin( _colorSine*Math.sqrt(2) ) * 0.5 + 0.5;
			_colorShift[2] = Math.sin( _colorSine*Math.sqrt(3) ) * 0.5 + 0.5;
		}
		
		private function onDoubleClick(event:MouseEvent):void
		{
			_targetScale *= 1.25;
			_targetX = _mousePos.x;
			_targetY = _mousePos.y;
		}
		
		
	}
	
}
