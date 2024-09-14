package code
{
	/*****************************************
	 * Interactivity3 :
	 * Demonstrates movement controlled by a joystick.
   * http://www.adobe.com/devnet/actionscript/samples/interactivity_3.html
   * -------------------
	 * See 3_joystick.fla
	 ****************************************/
	 
	import flash.events.Event;
	import flash.events.MouseEvent;
	import flash.display.MovieClip;
	
	public class Interactivity3 extends MovieClip
	{
		//*************************
		// Properties:
		
		public var initx:Number = 0;
		public var inity:Number = 0;
		public var tension:Number = .5;
		public var decay:Number = .5;
		public var xSpeed:Number = 0;
		public var dragging:Boolean = false;
		private var moved:Boolean = false;
		
		//*************************
		// Constructor:
		
		public function Interactivity3()
		{
			trace('Interactivity3 init');
			initx = joystick.x;
			inity = joystick.y;

			// Respond to mouse events
			joystick.addEventListener(MouseEvent.MOUSE_DOWN,dragPressHandler);
			stage.addEventListener(MouseEvent.MOUSE_UP,dragReleaseHandler);
		
			// Update screen every frame
			addEventListener(Event.ENTER_FRAME,enterFrameHandler);
		}
		
		//*************************
		// Event Handling:
		
		protected function dragPressHandler(event:MouseEvent):void
		{
			trace('dragPressHandler');
			dragging = true;
		}
		
		protected function dragReleaseHandler(event:MouseEvent):void
		{
			trace('dragReleaseHandler');
			dragging = false;
		}
		
		protected function enterFrameHandler(event:Event):void
		{
			with( joystick )
			{
				if( dragging ) 
				{
					// Calculate the angle 
					// of the joystick
					var angle = Math.atan2(root.mouseY-inity,root.mouseX-initx)/(Math.PI/180);
					rotation = angle;
					
					with( knob ) 
					{
						// Rotate the knob inversely to 
						// the rotation of the whole joystick
						rotation = -angle;
						
						// Drag the joystick but constrain it 
						// to a circle with a radius of 75
						x = parent.mouseX;
						if( x > 75 ){
							x = 75;
						}
					}
					with( beetle ) 
					{
						// Set rotation of beetle equal to 
						// the rotation of the joystick
						rotation = angle;
						
						// Loop to opposite side of the masked 
						// area when the beetle travels off-screen
						if( y < 0 ) {
							y = 231;
						}
						if( y > 231 ){
							y = 0;
						}
						if( x < 231 ){
							x = 465;
						}
						if (x > 370 && !moved) {
							moved = true;
							trace('wrap');
						}
						if( x > 465 ){
							x = 231;
						}
						// Move the beetle in proportion to how far 
						// the joystick is dragged from its center
						y += Math.sin(angle*(Math.PI/180))*(knob.x/8);
						x += Math.cos(angle*(Math.PI/180))*(knob.x/8);
					}
					// Scale the length of the joystick shaft
					shaft.width = (knob.x-shaft.x);
					shaft.alpha = 1;
				} 
				else{
					// Snap back to center when the joystick is released
					xSpeed = -knob.x*tension+(xSpeed*decay);
					knob.x += xSpeed;
					shaft.alpha = 0;
				}
			}
		}
	}
}