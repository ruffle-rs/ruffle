package
{
    
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.filters.DropShadowFilter;
	import flash.display.Stage;
	import flash.utils.ByteArray;
    
    public class Test extends Sprite
    {
        public const viewWidth:Number = 320;
        public const viewHeight:Number = 200;
        
        private var bitmap:Bitmap;
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
		
		private function dumpBytes(data: ByteArray) {
			var out = new Array();
			data.position = 0;
			
			for (var i = 0; i < data.length; i++) {
				out.push(data.readByte())
			};
		
			trace(out);
		}
        
        public function Test(stage:Stage)
        {
            stage3D = stage.stage3Ds[0];
            //stage3D.x = 0;
            //stage3D.y = 0;

            //Compile shaders
            vertexAssembly.assemble( Context3DProgramType.VERTEX, VERTEX_SHADER, 1, false );
            fragmentAssembly.assemble( Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1, false );		
			
            //Add event listener before requesting the context
            stage3D.addEventListener( Event.CONTEXT3D_CREATE, contextCreated );            
            stage3D.requestContext3D( Context3DRenderMode.AUTO );
		}
        
        //Note, context3DCreate event can happen at any time, such as when the hardware resources are taken by another process
        private function contextCreated( event:Event ):void
        {
            renderContext = Stage3D( event.target ).context3D;
            //trace( "3D driver: " + renderContext.driverInfo );

            //renderContext.enableErrorChecking = true; //Can slow rendering - only turn on when developing/testing
            renderContext.configureBackBuffer( viewWidth, viewHeight, 2, false );
            
            //Create vertex index list for the triangles
            var triangles:Vector.<uint> = Vector.<uint>( [ 0, 1, 2, 0, 3, 4 ] );
            indexList = renderContext.createIndexBuffer( triangles.length );
            indexList.uploadFromVector( triangles, 0, triangles.length );
            
            //Create vertexes
            const dataPerVertex:int = 8;
            var vertexData:Vector.<Number> = Vector.<Number>(
                [
                  // x, y, z w    r, g, b, w format
                     0, 0, 0, 1,   1, 1, 1, 1,
                    -1, 1, 0, 1,   0, 0,.5, 1,
                     1, 1, 0, 1,   0, 1, 1, 1,
                     1,-1, 0, 1,  .5, 0, 0, 1,
                    -1,-1, 0, 1,   1, 0, 0, 1
                ]
            );
            vertexes = renderContext.createVertexBuffer( vertexData.length/dataPerVertex, dataPerVertex );
            vertexes.uploadFromVector( vertexData, 0, vertexData.length/dataPerVertex );
            
            //Identify vertex data inputs for vertex program
            renderContext.setVertexBufferAt( 0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_4 ); //va0 is position
            renderContext.setVertexBufferAt( 1, vertexes, 4, Context3DVertexBufferFormat.FLOAT_4 ); //va1 is color
            
            //Upload programs to render context
            programPair = renderContext.createProgram();
            programPair.upload( vertexAssembly.agalcode, fragmentAssembly.agalcode );
            renderContext.setProgram( programPair );
            
            //Clear required before first drawTriangles() call
            renderContext.clear( .3,.3,.3 );
            
            
            //Draw the 2 triangles
            renderContext.drawTriangles( indexList, 0, 2 );
            
            //var renderedBitmapData:BitmapData = new BitmapData( viewWidth, viewHeight, true );
            //renderContext.drawToBitmapData( renderedBitmapData );
            
            renderContext.present();
            
			/*
            //Add to stage
            bitmap = new Bitmap( renderedBitmapData );
            this.addChild( bitmap );
            bitmap.x = 55;
            bitmap.y = 25;
            bitmap.filters = [new DropShadowFilter( 8, 235, .4 )];
			*/
        }
    }
}
