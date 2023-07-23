package
{
    import com.adobe.utils.AGALMiniAssembler;
    
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DBlendFactor;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.events.TimerEvent;
    import flash.geom.Rectangle;
    import flash.ui.Keyboard;
    import flash.utils.Timer;
    
	// Based on the example from https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display3D/Context3D.html#setScissorRectangle()
    public class Main extends Sprite
    {
        public const viewWidth:Number = 640;
        public const viewHeight:Number = 480;
        
        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;
        
        private const VERTEX_SHADER:String =
            "mov op, va0    \n" +    //copy position to output 
            "mov v0, va1"; //copy color to varying variable v0
        
        private const FRAGMENT_SHADER:String = 
            "mov oc, v0"; //Set the output color to the value interpolated from the three triangle vertices 

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var programPair:Program3D;
        
        private var scissorOn:Boolean = true;
        private var toggler:Timer = new Timer( 750 );
        
        public function Main()
        {            
            stage3D = this.stage.stage3Ds[0];
            stage3D.x = 10;
            stage3D.y = 10;

            //Add event listener before requesting the context
            stage3D.addEventListener( Event.CONTEXT3D_CREATE, contextCreated );            
            stage3D.requestContext3D( Context3DRenderMode.AUTO, "standard" );
            
            //Compile shaders
            vertexAssembly.assemble( Context3DProgramType.VERTEX, VERTEX_SHADER, 2 );
            fragmentAssembly.assemble( Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 2 );
            
            //Set up timer to turn scissoring on and off
            //toggler.addEventListener( TimerEvent.TIMER, toggleScissor );
        }
        
        //Note, context3DCreate event can happen at any time, such as when the hardware resources are taken by another process
        private function contextCreated( event:Event ):void
        {
            renderContext = Stage3D( event.target ).context3D;

            renderContext.enableErrorChecking = true; //Can slow rendering - only turn on when developing/testing
            renderContext.configureBackBuffer( viewWidth, viewHeight, 1, false );
            
            //Create vertex index list for the triangles
            var triangles:Vector.<uint> = Vector.<uint>( [  0, 3 , 2, 
                                                            0, 1, 3
                                                         ] );
            indexList = renderContext.createIndexBuffer( triangles.length );
            indexList.uploadFromVector( triangles, 0, triangles.length );
            
            //Create vertexes
            const dataPerVertex:int = 6;
            var vertexData:Vector.<Number> = Vector.<Number>(
                [
                  // x, y, z    r, g, b, a format 
                   -1, 1, 0,  1,0,0,
                    1, 1, 0,  0,0,1,
                   -1,-1, 0,  0,1,0,
                    1,-1, 0,  1,0,1
                ]
            );
            vertexes = renderContext.createVertexBuffer( vertexData.length/dataPerVertex, dataPerVertex );
            vertexes.uploadFromVector( vertexData, 0, vertexData.length/dataPerVertex );
            
            //Identify vertex data inputs for vertex program
            renderContext.setVertexBufferAt( 0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3 ); //va0 is position
            renderContext.setVertexBufferAt( 1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_3 ); //va1 is color
            
            //Upload programs to render context
            programPair = renderContext.createProgram();
            programPair.upload( vertexAssembly.agalcode, fragmentAssembly.agalcode );
            renderContext.setProgram( programPair );
            
            render();
            //toggler.start();
        }
        
        private function render():void
        {
            //Clear required before first drawTriangles() call
            renderContext.clear();

            // Set an invalid rectangle (zero width). Stage3D should
			// ignore this rectangle and render to the entire viewport
            var scissor:Rectangle = new Rectangle( 10, 20, 0, 300 );
			trace("Setting rect: " + scissor);
            if( scissorOn )    renderContext.setScissorRectangle( scissor ); //on
			
            else renderContext.setScissorRectangle( null ); //off
			
            //Draw the triangles
            renderContext.drawTriangles( indexList, 0, 2 );
            
            //Show the frame
            renderContext.present();
        }
        
        private function toggleScissor( event:Event ):void
        {
            scissorOn = !scissorOn;
            render();
        }
        
    }
}