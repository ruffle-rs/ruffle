package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DClearMask;
    import flash.display3D.Context3DCompareMode;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DStencilAction;
    import flash.display3D.Context3DTriangleFace;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.display.Stage;
    import flash.geom.Matrix3D;

    // Tests Context3D.setStencilActions and Context3D.setStencilReferenceValue.
    //
    // Layout (4 columns x 4 rows), each cell tests a different scenario.
    // A colored quad is drawn; where the stencil test passes, the color appears.
    // Where it fails, the black background shows through.
    //
    // Row 0: Compare modes (stencil buffer = 1, ref = 1)
    //   Col 0: ALWAYS     -> full green quad (always passes)
    //   Col 1: EQUAL      -> green only in center (where stencil was written)
    //   Col 2: NOT_EQUAL  -> green only at edges (where stencil is 0)
    //   Col 3: NEVER      -> nothing visible (never passes)
    //
    // Row 1: More compare modes (stencil buffer = 1, ref = 1)
    //   Col 0: LESS       -> nothing in center (1 < 1 is false)
    //   Col 1: LESS_EQUAL -> green in center (1 <= 1 is true)
    //   Col 2: GREATER    -> nothing in center (1 > 1 is false)
    //   Col 3: GREATER_EQ -> green in center (1 >= 1 is true)
    //
    // Row 2: Stencil actions (write stencil, apply action, then verify)
    //   Col 0: KEEP       -> stencil stays 1, test ==1 passes (green)
    //   Col 1: ZERO       -> stencil becomes 0, test ==0 passes (green)
    //   Col 2: SET(ref=5) -> stencil becomes 5, test ==5 passes (green)
    //   Col 3: INVERT     -> stencil 0xFF inverted to 0, test ==0 passes (green)
    //
    // Row 3: More actions + masks
    //   Col 0: INCR_SAT   -> stencil 1 incremented to 2, test ==2 passes (green)
    //   Col 1: DECR_WRAP  -> stencil 0 decremented wraps to 255, test ==255 passes (green)
    //   Col 2: readMask   -> stencil=0xFF, ref=0x0F, readMask=0x0F, EQUAL passes (green)
    //   Col 3: writeMask  -> writeMask=0x0F, write ref=0xFF -> only 0x0F written, test ==0x0F (green)

    public class Test extends Sprite {
        public const viewWidth:Number = 800;
        public const viewHeight:Number = 800;
        private const COLS:int = 4;
        private const ROWS:int = 4;

        private var stage3D:Stage3D;
        private var ctx:Context3D;

        private const VERTEX_SHADER:String =
            "m44 op, va0, vc0 \n" +
            "mov v0, va1";

        private const FRAGMENT_SHADER:String =
            "mov oc, v0";

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var programPair:Program3D;

        private var quadIndices:IndexBuffer3D;
        private var quadVertices:VertexBuffer3D;

        public function Test() {
            stage3D = stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D(Context3DRenderMode.AUTO);

            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 1);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1);
        }

        private function contextCreated(event:Event):void {
            ctx = Stage3D(event.target).context3D;
            ctx.enableErrorChecking = true;
            ctx.configureBackBuffer(viewWidth, viewHeight, 0, true);

            // Disable depth testing entirely - we only care about stencil
            ctx.setDepthTest(false, Context3DCompareMode.ALWAYS);

            setupGeometry();

            programPair = ctx.createProgram();
            programPair.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            ctx.setProgram(programPair);

            render();
        }

        private function setupGeometry():void {
            var triangles:Vector.<uint> = Vector.<uint>([0, 1, 2, 0, 2, 3]);
            quadIndices = ctx.createIndexBuffer(triangles.length);
            quadIndices.uploadFromVector(triangles, 0, triangles.length);

            const dataPerVertex:int = 7;
            var vertexData:Vector.<Number> = Vector.<Number>([
                -1, -1, 0,   0, 0, 0, 1,
                 1, -1, 0,   0, 0, 0, 1,
                 1,  1, 0,   0, 0, 0, 1,
                -1,  1, 0,   0, 0, 0, 1
            ]);
            quadVertices = ctx.createVertexBuffer(4, dataPerVertex);
            quadVertices.uploadFromVector(vertexData, 0, 4);

            ctx.setVertexBufferAt(0, quadVertices, 0, Context3DVertexBufferFormat.FLOAT_3);
            ctx.setVertexBufferAt(1, quadVertices, 3, Context3DVertexBufferFormat.FLOAT_4);
        }

        private function setColor(r:Number, g:Number, b:Number, a:Number = 1.0):void {
            const dataPerVertex:int = 7;
            var v:Vector.<Number> = Vector.<Number>([
                -1, -1, 0,  r, g, b, a,
                 1, -1, 0,  r, g, b, a,
                 1,  1, 0,  r, g, b, a,
                -1,  1, 0,  r, g, b, a
            ]);
            quadVertices.uploadFromVector(v, 0, 4);
        }

        // Returns a matrix that positions a unit quad into cell (col, row)
        private function cellMatrix(col:int, row:int):Matrix3D {
            var mat:Matrix3D = new Matrix3D();
            mat.appendScale(1.0 / COLS, 1.0 / ROWS, 1);
            mat.appendTranslation(
                -1.0 + (2.0 * col + 1.0) / COLS,
                 1.0 - (2.0 * row + 1.0) / ROWS,
                0
            );
            return mat;
        }

        // Draw full-cell quad (uses current stencil/color settings)
        private function drawCell(col:int, row:int):void {
            ctx.setProgramConstantsFromMatrix("vertex", 0, cellMatrix(col, row), true);
            ctx.drawTriangles(quadIndices, 0, 2);
        }

        // Draw a smaller (60%) quad in the center of a cell, to write stencil only
        private function drawStencilCenter(col:int, row:int):void {
            var mat:Matrix3D = new Matrix3D();
            mat.appendScale(0.6, 0.6, 1);
            var cell:Matrix3D = cellMatrix(col, row);
            mat.append(cell);
            ctx.setProgramConstantsFromMatrix("vertex", 0, mat, true);

            ctx.setColorMask(false, false, false, false);
            ctx.drawTriangles(quadIndices, 0, 2);
            ctx.setColorMask(true, true, true, true);
        }

        // Reset stencil buffer to 0 for the entire framebuffer (preserves color)
        private function clearStencil():void {
            ctx.clear(0, 0, 0, 0, 0, 0, Context3DClearMask.STENCIL);
        }

        // Disable stencil testing (set to always pass with no writes)
        private function disableStencil():void {
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
        }

        // Write stencil = refVal in the center of a cell (60% area)
        private function writeStencilCenter(col:int, row:int, refVal:uint):void {
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.SET,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(refVal, 0xFF, 0xFF);
            drawStencilCenter(col, row);
        }

        private function render():void {
            // Clear everything: black background, stencil=0
            ctx.clear(0, 0, 0, 1, 1, 0, Context3DClearMask.ALL);

            // ==================== ROW 0: Compare modes ====================
            var row0Modes:Array = [
                Context3DCompareMode.ALWAYS,
                Context3DCompareMode.EQUAL,
                Context3DCompareMode.NOT_EQUAL,
                Context3DCompareMode.NEVER
            ];
            var row0Colors:Array = [
                [0, 0.8, 0],    // green
                [0, 0.8, 0],    // green
                [0, 0.8, 0],    // green
                [0, 0.8, 0]     // green (but NEVER means nothing shows)
            ];
            for (var i:int = 0; i < 4; i++) {
                clearStencil();
                writeStencilCenter(i, 0, 1);

                ctx.setStencilActions(
                    Context3DTriangleFace.FRONT_AND_BACK,
                    row0Modes[i],
                    Context3DStencilAction.KEEP,
                    Context3DStencilAction.KEEP,
                    Context3DStencilAction.KEEP
                );
                ctx.setStencilReferenceValue(1, 0xFF, 0xFF);
                setColor(row0Colors[i][0], row0Colors[i][1], row0Colors[i][2]);
                drawCell(i, 0);
            }

            // ==================== ROW 1: More compare modes ====================
            var row1Modes:Array = [
                Context3DCompareMode.LESS,
                Context3DCompareMode.LESS_EQUAL,
                Context3DCompareMode.GREATER,
                Context3DCompareMode.GREATER_EQUAL
            ];
            var row1Colors:Array = [
                [0, 0.6, 0.8],
                [0, 0.6, 0.8],
                [0, 0.6, 0.8],
                [0, 0.6, 0.8]
            ];
            for (i = 0; i < 4; i++) {
                clearStencil();
                writeStencilCenter(i, 1, 1);

                ctx.setStencilActions(
                    Context3DTriangleFace.FRONT_AND_BACK,
                    row1Modes[i],
                    Context3DStencilAction.KEEP,
                    Context3DStencilAction.KEEP,
                    Context3DStencilAction.KEEP
                );
                ctx.setStencilReferenceValue(1, 0xFF, 0xFF);
                setColor(row1Colors[i][0], row1Colors[i][1], row1Colors[i][2]);
                drawCell(i, 1);
            }

            // ==================== ROW 2: Stencil actions ====================

            // Col 0: KEEP - write 1, apply KEEP, test ==1 -> center is green
            clearStencil();
            writeStencilCenter(0, 2, 1);
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(1, 0xFF, 0xFF);
            drawStencilCenter(0, 2); // stencil stays 1 (KEEP)
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(1, 0xFF, 0xFF);
            setColor(0.8, 0.2, 0.2);
            drawCell(0, 2);

            // Col 1: ZERO - write 1, apply ZERO, test ==0 -> center is green
            clearStencil();
            writeStencilCenter(1, 2, 1);
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.ZERO,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
            drawStencilCenter(1, 2); // stencil becomes 0 (ZERO)
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
            setColor(0.8, 0.8, 0.2);
            drawCell(1, 2);

            // Col 2: SET(ref=5) - write 1, apply SET with ref=5, test ==5 -> center is green
            clearStencil();
            writeStencilCenter(2, 2, 1);
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.SET,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(5, 0xFF, 0xFF);
            drawStencilCenter(2, 2); // stencil becomes 5 (SET with ref=5)
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(5, 0xFF, 0xFF);
            setColor(0.2, 0.8, 0.2);
            drawCell(2, 2);

            // Col 3: INVERT - write 0xFF, apply INVERT, test ==0 -> center is green
            clearStencil();
            writeStencilCenter(3, 2, 0xFF);
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.INVERT,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
            drawStencilCenter(3, 2); // stencil 0xFF inverted to 0x00
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
            setColor(0.6, 0.2, 0.8);
            drawCell(3, 2);

            // ==================== ROW 3: More actions + masks ====================

            // Col 0: INCREMENT_SATURATE - write 1, increment, test ==2 -> center is green
            clearStencil();
            writeStencilCenter(0, 3, 1);
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.INCREMENT_SATURATE,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
            drawStencilCenter(0, 3); // stencil 1 -> 2
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(2, 0xFF, 0xFF);
            setColor(0.8, 0.4, 0);
            drawCell(0, 3);

            // Col 1: DECREMENT_WRAP - stencil starts at 0, decrement wraps to 255, test ==255
            clearStencil();
            // Stencil is already 0 from clear
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.DECREMENT_WRAP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0, 0xFF, 0xFF);
            drawStencilCenter(1, 3); // stencil 0 -> 255
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(255, 0xFF, 0xFF);
            setColor(0.8, 0.6, 0);
            drawCell(1, 3);

            // Col 2: Read mask - write 0xFF, test with ref=0x0F and readMask=0x0F
            // (0x0F & 0x0F) == (0xFF & 0x0F) -> 0x0F == 0x0F -> EQUAL passes
            clearStencil();
            writeStencilCenter(2, 3, 0xFF);
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0x0F, 0x0F, 0xFF);
            setColor(0.4, 0.8, 0.4);
            drawCell(2, 3);

            // Col 3: Write mask - write with writeMask=0x0F and ref=0xFF
            // Only low nibble written, so stencil=0x0F. Test ==0x0F passes.
            clearStencil();
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.ALWAYS,
                Context3DStencilAction.SET,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0xFF, 0xFF, 0x0F);
            drawStencilCenter(3, 3); // only low nibble written -> stencil = 0x0F
            ctx.setStencilActions(
                Context3DTriangleFace.FRONT_AND_BACK,
                Context3DCompareMode.EQUAL,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP,
                Context3DStencilAction.KEEP
            );
            ctx.setStencilReferenceValue(0x0F, 0xFF, 0xFF);
            setColor(0.4, 0.4, 0.8);
            drawCell(3, 3);

            ctx.present();
        }
    }
}
