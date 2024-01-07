package {

    import flash.display.CapsStyle;
    import flash.display.JointStyle;
    import flash.display.LineScaleMode;
    import flash.display.MovieClip;
    import flash.display.Graphics;
    import flash.display.Shape;
    import flash.display.GraphicsPathCommand;
    import flash.display.GraphicsPath;
    import flash.display.IGraphicsData;
    import flash.display.GraphicsStroke;
    import flash.display.GraphicsSolidFill;
    import flash.geom.Matrix;
    import flash.display.GraphicsGradientFill;

    public class Test extends MovieClip {

        public function Test() {
            drawPath();
            drawGraphicsData();
        }

        function drawPath() {
            var shape:Shape = new Shape();

            // Complete nonsense in 'commands', including an invalid command
            var commands:Vector.<int> = new Vector.<int>();
            commands[0] = 2;
            commands[1] = 2;
            commands[2] = 100;
            var coords:Vector.<Number> = new Vector.<Number>(0);

            shape.graphics.beginFill(0x003366);
            // Flash ignores 'commands' when 'coords' is empty
            shape.graphics.drawPath(commands, coords);
            trace("Successfully called drawPath with empty coords and invalid commands");

            var star_commands:Vector.<int> = new Vector.<int>(5, true);

            star_commands[0] = GraphicsPathCommand.MOVE_TO;
            star_commands[1] = GraphicsPathCommand.LINE_TO;
            star_commands[2] = GraphicsPathCommand.LINE_TO;
            star_commands[3] = GraphicsPathCommand.LINE_TO;
            star_commands[4] = GraphicsPathCommand.LINE_TO;

            var star_coord:Vector.<Number> = new Vector.<Number>(10, true);
            star_coord[0] = 66; // x
            star_coord[1] = 10; // y
            star_coord[2] = 23;
            star_coord[3] = 127;
            star_coord[4] = 122;
            star_coord[5] = 50;
            star_coord[6] = 10;
            star_coord[7] = 49;
            star_coord[8] = 109;
            star_coord[9] = 127;

            shape.graphics.drawPath(star_commands, star_coord);

            var bad_commands:Vector.<int> = new Vector.<int>(5, true);

            bad_commands[0] = GraphicsPathCommand.MOVE_TO;
            bad_commands[1] = GraphicsPathCommand.LINE_TO;
            bad_commands[2] = GraphicsPathCommand.LINE_TO;
            bad_commands[3] = GraphicsPathCommand.LINE_TO;
            bad_commands[4] = GraphicsPathCommand.LINE_TO;

            var bad_coords:Vector.<Number> = new Vector.<Number>(9, true);
            bad_coords[0] = 66; // x
            bad_coords[1] = 40; // y
            bad_coords[2] = 23;
            bad_coords[3] = 167;
            bad_coords[4] = 122;
            bad_coords[5] = 90;
            bad_coords[6] = 40;
            bad_coords[7] = 99;
            bad_coords[8] = 109;
            // bad_coords[9] = 167;

            try {
                shape.graphics.drawPath(bad_commands, bad_coords);
            }
            catch (e) {
                trace("Caught error: " + e);
            }

            shape.y = 50;
            this.addChild(shape);
        }

        function drawGraphicsData() {
            var shape = new Shape();

            // Commands will be ignored by Flash since coords is empty
            var emptyCoords:Vector.<Number> = Vector.<Number>([]);
            var nonEmptyCommands = new Vector.<int>(1, true);
            nonEmptyCommands[0] = GraphicsPathCommand.MOVE_TO;

            var weirdPath:GraphicsPath = new GraphicsPath(nonEmptyCommands, emptyCoords);
            var weirdDrawing:Vector.<IGraphicsData> = new Vector.<IGraphicsData>(0, false);
            weirdDrawing[0] = weirdPath;

            // render the drawing
            shape.graphics.drawGraphicsData(weirdDrawing);
            trace("Successful weird drawing with drawGraphicsData");

            goodGraphicsDraw(shape);
            badGraphicsDraw(shape);

            shape.x = 150;
            this.addChild(shape);
            trace("Done");
        }

        private function goodGraphicsDraw(shape:Shape) {
            // establish the fill properties
            var myFill:GraphicsGradientFill = new GraphicsGradientFill();
            myFill.colors = [0xEEFFEE, 0x0000FF];
            myFill.matrix = new Matrix();
            myFill.matrix.createGradientBox(100, 100, 0);

            // establish the stroke properties
            var myStroke:GraphicsStroke = new GraphicsStroke(2);
            myStroke.fill = new GraphicsSolidFill(0x000000);

            // establish the path properties
            var pathCommands = new Vector.<int>(5, true);
            pathCommands[0] = GraphicsPathCommand.MOVE_TO;
            pathCommands[1] = GraphicsPathCommand.LINE_TO;
            pathCommands[2] = GraphicsPathCommand.LINE_TO;
            pathCommands[3] = GraphicsPathCommand.LINE_TO;
            pathCommands[4] = GraphicsPathCommand.LINE_TO;

            var pathCoordinates:Vector.<Number> = new Vector.<Number>(0, false);
            pathCoordinates.push(10, 10, 10, 100, 100, 100, 100, 10, 10, 10);

            var myPath:GraphicsPath = new GraphicsPath(pathCommands, pathCoordinates);

            // populate the IGraphicsData Vector array
            var myDrawing:Vector.<IGraphicsData> = new Vector.<IGraphicsData>(0, false);
            myDrawing[0] = myStroke;
            myDrawing[1] = myPath;

            // render the drawing
            shape.graphics.drawGraphicsData(myDrawing);
        }

        private function badGraphicsDraw(shape:Shape) {
            // establish the fill properties
            var myFill:GraphicsGradientFill = new GraphicsGradientFill();
            myFill.colors = [0xEEFFEE, 0x0000FF];
            myFill.matrix = new Matrix();
            myFill.matrix.createGradientBox(100, 100, 0);

            // establish the stroke properties
            var myStroke:GraphicsStroke = new GraphicsStroke(2);
            myStroke.fill = new GraphicsSolidFill(0x000000);

            // establish the path properties
            var pathCommands = new Vector.<int>(5, true);
            pathCommands[0] = GraphicsPathCommand.MOVE_TO;
            pathCommands[1] = GraphicsPathCommand.LINE_TO;
            pathCommands[2] = GraphicsPathCommand.LINE_TO;
            pathCommands[3] = GraphicsPathCommand.LINE_TO;
            pathCommands[4] = GraphicsPathCommand.LINE_TO;

            // Deliberately too short
            var pathCoordinates:Vector.<Number> = new Vector.<Number>(0, false);
            pathCoordinates.push(10, 10, 10, 100, 100, 100, 100, 10, 10);

            var myPath:GraphicsPath = new GraphicsPath(pathCommands, pathCoordinates);

            // populate the IGraphicsData Vector array
            var myDrawing:Vector.<IGraphicsData> = new Vector.<IGraphicsData>(0, false);
            myDrawing[0] = myStroke;
            myDrawing[1] = myPath;

            // render the drawing
            try {
                shape.graphics.drawGraphicsData(myDrawing);
            }
            catch (e) {
                trace("Caught error in drawGraphicsData: " + e);
            }
        }
    }

}
