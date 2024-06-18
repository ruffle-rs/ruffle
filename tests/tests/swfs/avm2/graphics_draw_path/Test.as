package {

import flash.display.GraphicsPath;
import flash.display.GraphicsPathCommand;
import flash.display.GraphicsPathWinding;
import flash.display.IGraphicsData;
import flash.display.MovieClip;
import flash.display.Graphics;
import flash.display.Shape;

[SWF(width="50", height="90", backgroundColor="#000000")]
public class Test extends MovieClip {
    private var shapeId:int = 0;

    public function Test() {
        testCases(function (g:Graphics, commands:Vector.<int>, data:Vector.<Number> = null, winding:String = "evenOdd"): void {
            g.drawPath(commands, data, winding);
        });
        testCases(function (g:Graphics, commands:Vector.<int>, data:Vector.<Number> = null, winding:String = "evenOdd"): void {
            g.drawGraphicsData(Vector.<IGraphicsData>([
                new GraphicsPath(commands, data, winding)
            ]));
        });
        smokeTest();
        trace("===== drawPath")
        testErrors(function (commands:Vector.<int>, data:Vector.<Number> = null, winding:String = "evenOdd"): void {
            new Shape().graphics.drawPath(commands, data, winding);
        });
        trace("===== GraphicsPath")
        testErrors(function (commands:Vector.<int>, data:Vector.<Number> = null, winding:String = "evenOdd"): void {
            new Shape().graphics.drawGraphicsData(Vector.<IGraphicsData>([
                new GraphicsPath(commands, data, winding)
            ]));
        });
    }

    private function testCases(draw: Function) {
        var g:Graphics;

        // 1. Basic rectangle
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            GraphicsPathCommand.NO_OP,
            GraphicsPathCommand.NO_OP,
            GraphicsPathCommand.NO_OP,
            GraphicsPathCommand.NO_OP,
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            0, 0,
            10, 0,
            10, 10,
            0, 10,
            0, 0
        ]));
        g.endFill();

        // 2. Too much data
        g = testShape().graphics;
        g.beginFill(0x0000FF);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            0, 0,
            10, 0,
            0, 10,
            10, 10,
            0, 0,
            5, 5,
            5, 5,
            5, 5,
            5, 5
        ]));
        g.endFill();

        // 3. Too much commands
        g = testShape().graphics;
        g.beginFill(0x00FFFF);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            10, 10,
            0, 0,
            10, 0,
            0, 10
        ]));
        g.endFill();

        // 4. Unknown commands
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            42,
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            5, 5,
            0, 0,
            10, 0,
            10, 10,
            0, 10,
            0, 0
        ]));
        g.endFill();

        // 5. String coord
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            "0", "0",
            "10", "0",
            "0", "10",
            "10", "10",
            "0", "0"
        ]));
        g.endFill();

        // 6. Non-integer coord
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            0, 0,
            "X", "Y",
            "X", "Y",
            0, 0,
            10, 0,
            10, 10,
            0, 10,
            0, 0
        ]));
        g.endFill();

        // 7. No data
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([]));
        g.endFill();

        // 8. No commands
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([]), Vector.<Number>([
            0, 0,
            10, 0,
            0, 10,
            10, 10,
            0, 0
        ]));
        g.endFill();

        // 9. Even-odd winding
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            0, 0,
            0, 10,
            10, 10,
            10, 0,
            0, 0,
            0, 10,
            10, 0,
            10, 10,
            0, 0
        ]), GraphicsPathWinding.EVEN_ODD);
        g.endFill();

        // 10. Non-zero winding
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<int>([
            GraphicsPathCommand.MOVE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO,
            GraphicsPathCommand.LINE_TO
        ]), Vector.<Number>([
            0, 0,
            0, 10,
            10, 10,
            10, 0,
            0, 0,
            0, 10,
            10, 0,
            10, 10,
            0, 0
        ]), GraphicsPathWinding.NON_ZERO);
        g.endFill();
    }

    private function testShape():Shape {
        var child:Shape = new Shape();
        child.x = 10 * (shapeId % 5);
        child.y = 10 * Math.floor(shapeId / 5);
        ++shapeId;
        addChild(child);
        return child;
    }

    private function smokeTest() {
        var child:Shape = new Shape();
        child.x = 0;
        child.y = 40;
        addChild(child);
        var g = child.graphics;
        g.beginFill(0xFF0000);
        for each (var dy in [0, 10, 20, 30, 40]) {
            g.drawPath(Vector.<int>([
                GraphicsPathCommand.MOVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO
            ]), Vector.<Number>([
                0,  dy + 10,
                10, dy + 0,
                10, dy + 10,
                10, dy + 10,
                20, dy + 0,
                20, dy + 10,
                20, dy + 10,
                30, dy + 0,
                30, dy + 10,
                30, dy + 10,
                40, dy + 0,
                40, dy + 10,
                40, dy + 10,
                50, dy + 0,
                50, dy + 10
            ]));
        }
        g.endFill();
        g.beginFill(0x0000FF);
        for each (var dy in [0, 10, 20, 30, 40]) {
            g.drawPath(Vector.<int>([
                GraphicsPathCommand.MOVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO
            ]), Vector.<Number>([
                0,  dy + 0,
                0,  dy + 10,
                10, dy + 0,
                10, dy + 0,
                10, dy + 10,
                20, dy + 0,
                20, dy + 0,
                20, dy + 10,
                30, dy + 0,
                30, dy + 0,
                30, dy + 10,
                40, dy + 0,
                40, dy + 0,
                40, dy + 10,
                50, dy + 0
            ]));
        }
        g.endFill();

        for each (var i in [0, 1]) {
            var dy = i ? 30 : 0;
            var windingRule = i ? GraphicsPathWinding.NON_ZERO : GraphicsPathWinding.EVEN_ODD;
            g.beginFill(i ? 0xFFFF00 : 0x00FFFF);
            g.drawPath(Vector.<int>([
                GraphicsPathCommand.WIDE_MOVE_TO,
                GraphicsPathCommand.CUBIC_CURVE_TO,
                GraphicsPathCommand.WIDE_LINE_TO,
                GraphicsPathCommand.CURVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.CUBIC_CURVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.CUBIC_CURVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.CUBIC_CURVE_TO,
                GraphicsPathCommand.CURVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.NO_OP
            ]), Vector.<Number>([
                -1, dy + -1, 10, dy + 5,
                10, dy + 0,  20, dy + 0,  20, dy + 5,
                -1, dy + -1, 20, dy + 10,
                20, dy + 15, 30, dy + 15,
                40, dy + 15,
                50, dy + 15, 50, dy + 10, 40, dy + 10,
                10, dy + 10,
                0,  dy + 10, 0,  dy + 5,  10, dy + 5,
                30, dy + 5,
                40, dy + 5,  40, dy + 20, 30, dy + 20,
                10, dy + 20, 10, dy + 10,
                10, dy + 5,
                -1, dy + -1
            ]), windingRule);
            g.endFill();
        }
    }

    private function logError(f:Function):void {
        try {
            f();
            trace("  No error thrown");
        } catch (e:*) {
            trace("  Error thrown: " + e);
        }
    }

    private function testErrors(draw: Function):void {
        trace("Odd number of data coordinates")
        logError(function ():void {
            draw(Vector.<int>([
                GraphicsPathCommand.MOVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO
            ]), Vector.<Number>([
                0, 0,
                0, 10,
                10
            ]));
        });

        trace("Even number of data coordinates")
        logError(function ():void {
            draw(Vector.<int>([
                GraphicsPathCommand.MOVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO
            ]), Vector.<Number>([
                0, 0,
                0, 10,
                10, 0
            ]));
        });

        trace("Odd number of data coordinates with no commands")
        logError(function ():void {
            draw(Vector.<int>([]), Vector.<Number>([
                0, 0,
                0, 10,
                10
            ]));
        });

        trace("Odd number of superfluous data coordinates")
        logError(function ():void {
            draw(Vector.<int>([
                GraphicsPathCommand.MOVE_TO,
                GraphicsPathCommand.LINE_TO,
                GraphicsPathCommand.LINE_TO
            ]), Vector.<Number>([
                0, 0,
                0, 10,
                10, 0,
                10
            ]));
        });

        trace("Unknown commands")
        logError(function ():void {
            draw(Vector.<int>([
                7,
                14,
                39,
                -1
            ]), Vector.<Number>([
                0, 0,
                0, 10,
                10, 0,
                10, 10
            ]));
        });
    }
}

}
