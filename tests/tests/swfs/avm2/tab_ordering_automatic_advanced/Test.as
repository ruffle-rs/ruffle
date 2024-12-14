package {

import flash.display.Sprite;
import flash.display.MovieClip;
import flash.events.KeyboardEvent;

[SWF(width="1000", height="1000", backgroundColor="#000000")]
public class Test extends MovieClip {
    // This is the originally focused sprite.
    // Upon pressing tab, the focus will switch to other sprites.
    // By selecting the positions of others we can verify
    // which object is preferred by the automatic tab ordering.
    var origin:Sprite;

    var others:Array = [];

    // Tab collectors are used to prevent the order from performing a cycle.
    // They are placed far outside the visible area and "collect" the focus.
    var tabCollectors:Array = [];

    var nextTestStage:int = 0;
    var testStages:Array = [];

    public function Test() {
        super();

        // Set up collectors
        for each (var x in [-100000, 0, 100000]) {
            for each (var y in [-100000, 0, 100000]) {
                if (x == 0 && y == 0) continue;
                var tabCollector = newSprite();
                tabCollector.x = x;
                tabCollector.y = y;
                tabCollector.addEventListener("focusIn", function (evt:*):void {
                    trace("Collected the tab at " + evt.target.x + ", " + evt.target.y);
                    nextStage();
                });
                tabCollectors.push(tabCollector);
                stage.addChild(tabCollector);
            }
        }

        // Set up origin
        origin = newSprite();
        origin.x = 500;
        origin.y = 500;

        stage.addChild(origin);

        setUpTestCases();
        nextStage();

        stage.addEventListener("keyDown", function (evt:KeyboardEvent):void {
            if (evt.keyCode == 27) {
                stage.focus = origin;
            }
        });
    }

    function newSprite() {
        var sprite:Sprite = new Sprite();
        sprite.graphics.beginFill(0xFF00FF);
        sprite.graphics.drawRect(0, 0, 10, 10);
        sprite.tabEnabled = true;
        return sprite;
    }

    function nextStage() {
        // Clean up
        for each (var other in others) {
            other.tabEnabled = false;
            stage.removeChild(other);
        }
        others = [];

        if (nextTestStage >= testStages.length) {
            origin.tabEnabled = false;
            for each (var tabCollector in tabCollectors) {
                tabCollector.tabEnabled = false;
            }
            stage.focus = null;
            trace("===== Finished");
            return;
        }

        var config:* = testStages[nextTestStage];
        if (config["message"]) {
            trace("===== Stage " + nextTestStage + " (" + config["message"] + ")");
        } else {
            trace("===== Stage " + nextTestStage);
        }
        ++nextTestStage;

        var othersConfig = config["others"];
        var i = 0;
        for each (var otherConfig in othersConfig) {
            var other = newSprite();
            other.x = 500 + otherConfig["dx"];
            other.y = 500 + otherConfig["dy"];
            if (otherConfig["scale"]) {
                other.scaleX = otherConfig["scale"];
                other.scaleY = otherConfig["scale"];
            }
            other.name = "other" + (i++);
            other.addEventListener("focusIn", function (evt:*):void {
                trace("Focused " + evt.target.name + " at dx=" + (evt.target.x - 500) + ", dy=" + (evt.target.y - 500));
            });
            stage.addChild(other);
            others.push(other);
        }

        stage.focus = origin;
    }

    function setUpTestCases() {
        testStages.push({
            "message": "IV quadrant preference over y = -(x - p) / 6",
            "others": [
                {"dx": 10, "dy": 0},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 59, "dy": 0},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": 0},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 61, "dy": 0},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 119, "dy": 0},
                {"dx": 0, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 120, "dy": 0},
                {"dx": 0, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 121, "dy": 0},
                {"dx": 0, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 219, "dy": 100},
                {"dx": 100, "dy": 120}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 220, "dy": 100},
                {"dx": 100, "dy": 120}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 221, "dy": 100},
                {"dx": 100, "dy": 120}
            ]
        });

        testStages.push({
            "message": "III/IV quadrant preference over y = -(x - p) / 6",
            "others": [
                {"dx": 59, "dy": 0},
                {"dx": -60, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": 0},
                {"dx": -60, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 61, "dy": 0},
                {"dx": -60, "dy": 20}
            ]
        });

        testStages.push({
            "message": "III quadrant preference over y = -(x - p) / 6",
            "others": [
                {"dx": -61, "dy": 100},
                {"dx": -120, "dy": 110}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -60, "dy": 100},
                {"dx": -120, "dy": 110}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -59, "dy": 100},
                {"dx": -120, "dy": 110}
            ]
        });

        testStages.push({
            "message": "I/IV quadrant preference over y = -(x - p) / 6",
            "others": [
                {"dx": 259, "dy": -5},
                {"dx": 200, "dy": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 260, "dy": -5},
                {"dx": 200, "dy": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 261, "dy": -5},
                {"dx": 200, "dy": 5}
            ]
        });

        testStages.push({
            "message": "I quadrant preference over y = -(x - p) / 6",
            "others": [
                {"dx": 259, "dy": -15},
                {"dx": 200, "dy": -5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 260, "dy": -15},
                {"dx": 200, "dy": -5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 261, "dy": -15},
                {"dx": 200, "dy": -5}
            ]
        });

        testStages.push({
            "message": "I/III quadrant preference over y = -(x - p) / 6",
            "others": [
                {"dx": 119, "dy": -10},
                {"dx": -120, "dy": 30}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 120, "dy": -10},
                {"dx": -120, "dy": 30}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 121, "dy": -10},
                {"dx": -120, "dy": 30}
            ]
        });

        testStages.push({
            "message": "Cutoff over y < x / 6 around 60,-10",
            "others": [
                {"dx": 60, "dy": -10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 59, "dy": -10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 61, "dy": -10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": -9}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": -11}
            ]
        });

        testStages.push({
            "message": "Cutoff over y < x / 6 around 0,0",
            "others": [
                {"dx": 0, "dy": 0}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -1, "dy": 0}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 1, "dy": 0}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": -1}
            ]
        });

        testStages.push({
            "message": "Cutoff over y < x / 6 around -120,20",
            "others": [
                {"dx": -120, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -121, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -119, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 21}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 19}
            ]
        });

        testStages.push({
            "message": "Scale should not influence the preference",
            "others": [
                {"dx": 59, "dy": 0, "scale": 0.5},
                {"dx": 0, "dy": 10, "scale": 2}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": 0, "scale": 0.5},
                {"dx": 0, "dy": 10, "scale": 2}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 61, "dy": 0, "scale": 0.5},
                {"dx": 0, "dy": 10, "scale": 2}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 59, "dy": 0, "scale": 3},
                {"dx": 0, "dy": 10, "scale": 0.1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": 0, "scale": 3},
                {"dx": 0, "dy": 10, "scale": 0.1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 61, "dy": 0, "scale": 3},
                {"dx": 0, "dy": 10, "scale": 0.1}
            ]
        });

        testStages.push({
            "message": "Scale should not influence the cutoff line",
            "others": [
                {"dx": -120, "dy": 20, "scale": 3}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -121, "dy": 20, "scale": 3}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -119, "dy": 20, "scale": 3}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 21, "scale": 3}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 19, "scale": 3}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 20, "scale": 0.1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -121, "dy": 20, "scale": 0.1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -119, "dy": 20, "scale": 0.1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 21, "scale": 0.1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -120, "dy": 19, "scale": 0.1}
            ]
        });

        testStages.push({
            "message": "Same preference vs order",
            "others": [
                {"dx": 60, "dy": 0},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 10},
                {"dx": 60, "dy": 0}
            ]
        });

        testStages.push({
            "message": "Same position vs order",
            "others": [
                {"dx": 60, "dy": 10},
                {"dx": 60, "dy": 10}
            ]
        });
    }
}
}
