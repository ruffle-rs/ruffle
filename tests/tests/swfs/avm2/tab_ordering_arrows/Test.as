package {

import flash.display.Sprite;
import flash.display.MovieClip;
import flash.events.KeyboardEvent;
import flash.ui.Keyboard;

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
    var arrowDirection:uint = 0;

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
            if (evt.keyCode == Keyboard.ESCAPE) {
                stage.focus = origin;
            }
            if (evt.keyCode == Keyboard.NUMBER_1) {
                arrowDirection += 1;
                nextTestStage = 0;
                origin.tabEnabled = true;
                for each (var tabCollector in tabCollectors) {
                    tabCollector.tabEnabled = true;
                }
                trace("===== Changing arrow: " + arrowDirection);
                nextStage();
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
            trace("===== Stage " + nextTestStage + " (direction " + arrowDirection + ", " + config["message"] + ")");
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
            if (otherConfig["w"]) {
                other.width = otherConfig["w"];
            }
            if (otherConfig["h"]) {
                other.height = otherConfig["h"];
            }
            if (otherConfig["tabIndex"]) {
                other.tabIndex = otherConfig["tabIndex"];
            }
            other.name = "other" + (i++);
            other.addEventListener("focusIn", function (evt:*):void {
                trace("Focused " + evt.target.name + " at dx=" + (evt.target.x - 500) + ", dy=" + (evt.target.y - 500)
                        + ", w=" + evt.target.width + ", h=" + evt.target.height);
            });

            if (arrowDirection == 0) { // Down
                // default
            } else if (arrowDirection == 1) { // Up
                other.x = 1000 - other.x - other.width;
                other.y = 1000 - other.y - other.height;
            } else if (arrowDirection == 2) { // Left
                var tmp = other.x;
                other.x = other.y;
                other.y = tmp;
                var tmp = other.scaleX;
                other.scaleX = other.scaleY;
                other.scaleY = tmp;
                other.x = 1000 - other.x - other.width;
                other.y = 1000 - other.y - other.height;
            } else if (arrowDirection == 3) { // Right
                var tmp = other.x;
                other.x = other.y;
                other.y = tmp;
                var tmp = other.scaleX;
                other.scaleX = other.scaleY;
                other.scaleY = tmp;
            }

            stage.addChild(other);
            others.push(other);
        }

        if (config["origin_w"]) {
            origin.width = config["origin_w"];
        } else {
            origin.width = 10;
        }
        if (config["origin_h"]) {
            origin.height = config["origin_h"];
        } else {
            origin.height = 10;
        }
        if (arrowDirection == 1 || arrowDirection == 2) {
            origin.scaleX = -1;
            origin.scaleY = -1;
        } else {
            origin.scaleX = 1;
            origin.scaleY = 1;
        }

        stage.focus = origin;
    }

    function setUpTestCases() {
        testStages.push({
            "message": "Initial",
            "others": [
                {"dx": 0, "dy": 20}
            ]
        });

        testStages.push({
            "message": "General direction",
            "others": [
                {"dx": 50, "dy": 0}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -50, "dy": 0}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 50}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": -50}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -50, "dy": 50}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": -50}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -50, "dy": -50}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Overlapping bounds",
            "others": [
                {"dx": 0, "dy": 0, "w": 5, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 5, "dy": 0, "w": 5, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 5, "dy": 5, "w": 5, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 5, "w": 5, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 0, "w": 10, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 5, "w": 10, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 0, "w": 5, "h": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 5, "dy": 0, "w": 5, "h": 10}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Specific direction",
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": 0}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": 1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": 1, "h": 9}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": 1, "h": 8}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": -10, "h": 19}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": -10, "h": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 50, "dy": 50},
                {"dx": 5, "dy": -10, "h": 21}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": -10, "h": 30},
                {"dx": -20, "dy": -10, "h": 30}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Same distance, behind origin",
            "others": [
                {"dx": -5, "dy": 10},
                {"dx": 5, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 5, "dy": 10},
                {"dx": -5, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 2, "dy": 10},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 10},
                {"dx": 2, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 10, "dy": 10},
                {"dx": -10, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -10, "dy": 10},
                {"dx": 10, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -7, "dy": 30},
                {"dx": 8, "dy": 30}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 8, "dy": 30},
                {"dx": -7, "dy": 30}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Same distance, different sizes, behind origin",
            "others": [
                {"dx": -5, "dy": 30, "h": 10},
                {"dx": 5, "dy": 30, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -5, "dy": 30, "h": 10},
                {"dx": 5, "dy": 35, "h": 5}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Different distance, behind origin",
            "others": [
                {"dx": 0, "dy": 11},
                {"dx": 0, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 10},
                {"dx": 0, "dy": 11}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 30},
                {"dx": -8, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -8, "dy": 30},
                {"dx": 0, "dy": 10}
            ]
        });

        testStages.push({
            "message": "Behind origin preference",
            "others": [
                {"dx": -11, "dy": 10},
                {"dx": -10, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -10, "dy": 10},
                {"dx": -11, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -10, "dy": 40},
                {"dx": -11, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 10, "dy": 10},
                {"dx": 11, "dy": 40}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 10, "dy": 40},
                {"dx": 11, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 10, "dy": 10},
                {"dx": 11, "dy": 40}
            ]
        });

        testStages.push({
            "message": "Size vs distance behind",
            "others": [
                {"dx": 0, "dy": -1, "h": 11},
                {"dx": 0, "dy": -2, "h": 12}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": -2, "h": 12},
                {"dx": 0, "dy": -1, "h": 11}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 20, "h": 10},
                {"dx": 0, "dy": 25, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 25, "h": 5},
                {"dx": 0, "dy": 20, "h": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 20, "h": 10},
                {"dx": 0, "dy": 20, "h": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 20, "h": 5},
                {"dx": 0, "dy": 20, "h": 10}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Behind vs wings",
            "others": [
                {"dx": 0, "dy": 90},
                {"dx": -30, "dy": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 90},
                {"dx": 30, "dy": 5}
            ]
        });

        testStages.push({
            "message": "Not behind vs wings",
            "others": [
                {"dx": 20, "dy": 20},
                {"dx": -80, "dy": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": 20},
                {"dx": 80, "dy": 5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": 20},
                {"dx": -80, "dy": -5}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": 20},
                {"dx": 80, "dy": -5}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Not behind origin",
            "others": [
                {"dx": -11, "dy": 10},
                {"dx": 11, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 11, "dy": 10},
                {"dx": -11, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -12, "dy": 10},
                {"dx": 12, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 12, "dy": 10},
                {"dx": -12, "dy": 10}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 25, "dy": 20},
                {"dx": 20, "dy": 25}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": 25},
                {"dx": 25, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 26, "dy": 20},
                {"dx": 20, "dy": 25}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": 25},
                {"dx": 26, "dy": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 24, "dy": 20},
                {"dx": 20, "dy": 25}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 20, "dy": 25},
                {"dx": 24, "dy": 20}
            ]
        });

        testStages.push({
            "others": [
                {"dx": 60, "dy": 210},
                {"dx": 160, "dy": 140}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": 210},
                {"dx": 160, "dy": 155}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -60, "dy": 210},
                {"dx": -160, "dy": 140}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -60, "dy": 210},
                {"dx": -160, "dy": 155}
            ]
        });

        testStages.push({
            "others": [
                {"dx": 60, "dy": 210, "w": 20, "h": 20},
                {"dx": 160, "dy": 140, "w": 20, "h": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 60, "dy": 210, "w": 20, "h": 20},
                {"dx": 160, "dy": 155, "w": 20, "h": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -70, "dy": 210, "w": 20, "h": 20},
                {"dx": -170, "dy": 140, "w": 20, "h": 20}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -70, "dy": 210, "w": 20, "h": 20},
                {"dx": -170, "dy": 155, "w": 20, "h": 20}
            ]
        });

        // =============================================

        testStages.push({
            "message": "Tab index",
            "others": [
                {"dx": 0, "dy": 30, "tab_index": 1},
                {"dx": 0, "dy": 60, "tab_index": 2}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 0, "dy": 30, "tab_index": 2},
                {"dx": 0, "dy": 60, "tab_index": 1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 5, "dy": 30, "tab_index": 2},
                {"dx": -5, "dy": 30, "tab_index": 1}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -5, "dy": 30, "tab_index": 1},
                {"dx": 5, "dy": 30, "tab_index": 2}
            ]
        });
        testStages.push({
            "others": [
                {"dx": 5, "dy": 30, "tab_index": 1},
                {"dx": -5, "dy": 30, "tab_index": 2}
            ]
        });
        testStages.push({
            "others": [
                {"dx": -5, "dy": 30, "tab_index": 2},
                {"dx": 5, "dy": 30, "tab_index": 1}
            ]
        });
    }
}
}
