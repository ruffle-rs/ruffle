package {

import flash.display.GraphicsTrianglePath;
import flash.display.IGraphicsData;
import flash.display.MovieClip;
import flash.display.Graphics;
import flash.display.Shape;

[SWF(width="50", height="110", backgroundColor="#000000")]
public class Test extends MovieClip {
    private var shapeId:int = 0;

    public function Test() {
        testCases(function (g:Graphics, vertices:Vector.<Number>, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none"): void {
            g.drawTriangles(vertices, indices, uvtData, culling);
        });
        testCases(function (g:Graphics, vertices:Vector.<Number>, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none"): void {
            g.drawGraphicsData(Vector.<IGraphicsData>([
                new GraphicsTrianglePath(vertices, indices, uvtData, culling)
            ]));
        });
        smokeTest();
        trace("===== drawTriangles")
        testErrors(function (vertices:Vector.<Number>, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none"): void {
            new Shape().graphics.drawTriangles(vertices, indices, uvtData, culling);
        });
        trace("===== GraphicsTrianglePath")
        testErrors(function (vertices:Vector.<Number>, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none"): void {
            new Shape().graphics.drawGraphicsData(Vector.<IGraphicsData>([
                new GraphicsTrianglePath(vertices, indices, uvtData, culling)
            ]));
        });
    }

    private function testCases(draw: Function) {
        var g:Graphics;

        // 1. One call, two triangles
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            0, 0,
            10, 0,
            10, 10,
            0, 0,
            0, 10,
            10, 10
        ]));
        g.endFill();

        // 2. Two triangles using indices
        g = testShape().graphics;
        g.beginFill(0x0000FF);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            0, 2, 3,
            0, 1, 3
        ]));
        g.endFill();

        // 3. Index overflow
        g = testShape().graphics;
        g.beginFill(0x00FFFF);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            0, 2, 3,
            0, 6, 7,
            0, 1, 3
        ]));
        g.endFill();

        // 4. Degenerate triangle
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            0, 0
        ]));
        g.endFill();

        // 5. Negative culling
        g = testShape().graphics;
        g.beginFill(0xFF00FF);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            0, 2, 3,
            0, 1, 3
        ]), null, "negative");
        g.endFill();

        // 6. Positive culling
        g = testShape().graphics;
        g.beginFill(0xFFFF00);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            0, 2, 3,
            0, 1, 3
        ]), null, "positive");
        g.endFill();

        // 7. String coord
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            "0", "0",
            "0", "10",
            "10", "0",
            "10", "10"
        ]), Vector.<int>([
            0, 1, 3
        ]));
        g.endFill();

        // 7. Non-integer coord
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            "0", "0",
            "0", "10",
            "10", "0",
            "10", "10",
            "X", "Y",
            "X", "Y",
            "X", "Y",
            "X", "Y"
        ]), Vector.<int>([
            4, 5, 7,
            0, 1, 3
        ]));
        g.endFill();

        // 9. String index
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            "0", "1", "3"
        ]));
        g.endFill();

        // 10. Non-integer index
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            "A", "B", "C",
            0, 1, 3
        ]));
        g.endFill();

        // 11. String coords without indices
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            "0", "0",
            "0", "10",
            "10", "0"
        ]));
        g.endFill();

        // 12. Non-integer coords without indices
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            "0", "0",
            "0", "10",
            "10", "0",
            "X", "Y",
            "X", "Y",
            "X", "Y"
        ]));
        g.endFill();

        // 13. Winding behavior
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            0, 0,
            0, 10,
            10, 10
        ]));
        g.endFill();

        // 14. Winding behavior with indices
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([
            0, 0,
            0, 10,
            10, 0,
            10, 10
        ]), Vector.<int>([
            0, 1, 2,
            0, 1, 3
        ]));
        g.endFill();

        // 15. No triangles
        g = testShape().graphics;
        g.beginFill(0xFF0000);
        draw(g, Vector.<Number>([]));
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
        child.y = 60;
        addChild(child);
        var g = child.graphics;
        var coords = [];
        for each (var y in [0, 10, 20, 30, 40, 50]) {
            for each (var x in [0, 10, 20, 30, 40, 50]) {
                coords.push(x);
                coords.push(y);
            }
        }
        g.beginFill(0xFF0000);
        g.drawTriangles(Vector.<Number>(coords), Vector.<int>([
            0, 1, 6,
            1, 2, 7,
            2, 3, 8,
            3, 4, 9,
            4, 5, 10,

            6, 7, 12,
            7, 8, 13,
            8, 9, 14,
            9, 10, 15,
            10, 11, 16,

            12, 13, 18,
            13, 14, 19,
            14, 15, 20,
            15, 16, 21,
            16, 17, 22,

            18, 19, 24,
            19, 20, 25,
            20, 21, 26,
            21, 22, 27,
            22, 23, 28,

            24, 25, 30,
            25, 26, 31,
            26, 27, 32,
            27, 28, 33,
            28, 29, 34
        ]));
        g.endFill();
        g.beginFill(0x0000FF);
        g.drawTriangles(Vector.<Number>(coords), Vector.<int>([
            0, 1, 7,
            1, 2, 8,
            2, 3, 9,
            3, 4, 10,
            4, 5, 11,

            6, 7, 13,
            7, 8, 14,
            8, 9, 15,
            9, 10, 16,
            10, 11, 17,

            12, 13, 19,
            13, 14, 20,
            14, 15, 21,
            15, 16, 22,
            16, 17, 23,

            18, 19, 25,
            19, 20, 26,
            20, 21, 27,
            21, 22, 28,
            22, 23, 29,

            24, 25, 31,
            25, 26, 32,
            26, 27, 33,
            27, 28, 34,
            28, 29, 35
        ]));
        g.endFill();
        g.beginFill(0x00FF00);
        g.drawTriangles(Vector.<Number>(coords), Vector.<int>([
            1, 12, 7,
            2, 13, 14,
            8, 16, 3,
            5, 3, 11,
            6, 17, 11,
            9, 10, 15
        ]));
        g.endFill();
        g.beginFill(0xFFFF00);
        g.drawTriangles(Vector.<Number>(coords), Vector.<int>([
            18 + 1, 18 + 12, 18 + 7,
            18 + 2, 18 + 13, 18 + 14,
            18 + 8, 18 + 16, 18 + 3,
            18 + 5, 18 + 3, 18 + 11,
            18 + 6, 18 + 17, 18 + 11,
            18 + 9, 18 + 10, 18 + 15
        ]), null, "positive");
        g.endFill();

        g.beginFill(0x00FFFF);
        g.drawTriangles(Vector.<Number>(coords), Vector.<int>([
            12, 13, 18,
            13, 18, 19,
            13, 14, 19,
            14, 19, 20,
            14, 15, 20,
            15, 20, 21,
            15, 16, 21,
            16, 21, 22,
            16, 17, 22,
            17, 22, 23
        ]), null, "positive");
        g.endFill();
        g.beginFill(0xFF00FF);
        g.drawTriangles(Vector.<Number>(coords), Vector.<int>([
            12, 13, 18,
            13, 18, 19,
            13, 14, 19,
            14, 19, 20,
            14, 15, 20,
            15, 20, 21,
            15, 16, 21,
            16, 21, 22,
            16, 17, 22,
            17, 22, 23
        ]), null, "negative");
        g.endFill();
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
        trace("0 coordinates")
        logError(function ():void {
            draw(Vector.<Number>([]));
        });

        trace("1 coordinate")
        logError(function ():void {
            draw(Vector.<Number>([
                0
            ]));
        });

        trace("2 coordinates")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1
            ]));
        });

        trace("3 coordinates")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2
            ]));
        });

        trace("4 coordinates")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2, 2
            ]));
        });

        trace("5 coordinates")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2, 2, 4
            ]));
        });

        trace("6 coordinates")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1,
                2, 2,
                3, 4
            ]));
        });

        trace("0 coordinates with indices")
        logError(function ():void {
            draw(Vector.<Number>([]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("1 coordinate with indices")
        logError(function ():void {
            draw(Vector.<Number>([
                0
            ]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("2 coordinates with indices")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1
            ]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("3 coordinates with indices")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2
            ]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("4 coordinates with indices")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2, 2
            ]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("5 coordinates with indices")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2, 2, 4
            ]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("6 coordinates with indices")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1,
                2, 2,
                3, 4
            ]), Vector.<int>([
                0, 1, 2
            ]));
        });

        trace("Unknown culling")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2
            ]), null, null, "what");
        });

        trace("Null culling")
        logError(function ():void {
            draw(Vector.<Number>([
                0, 1, 2
            ]), null, null, null);
        });

        trace("String coords")
        logError(function ():void {
            draw(Vector.<Number>([
                "0", "10",
                "10", "10",
                "10", "0"
            ]));
        });

        trace("Non-integer coords")
        logError(function ():void {
            draw(Vector.<Number>([
                "X", "Y",
                "X", "Y",
                "X", "Y"
            ]));
        });
    }
}

}
