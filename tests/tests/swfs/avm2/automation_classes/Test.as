package  {
    
import flash.display.Sprite;
import flash.automation.*;
import flash.events.Event;
import flash.events.EventDispatcher;


public class Test extends Sprite {

    public function Test() {
        super();

        var o:*;

        trace("=== Testing AutomationAction");
        try {
            o = new AutomationAction("hello");
        } catch(e) { trace(e); }
        o = new AutomationAction();
        testProp(o, "type");
        trace("");

        trace("=== Testing KeyboardAutomationAction");
        try {
            o = new KeyboardAutomationAction();
        } catch(e) { trace(e); }
        o = new KeyboardAutomationAction("hello");
        trace("KeyboardAutomationAction is AutomationAction:", o is AutomationAction);
        trace("== with default values");
        testProp(o, "type");
        testProp(o, "keyCode");
        trace("== with provided values");
        o = new KeyboardAutomationAction("hello", 12345.6789);
        testProp(o, "type");
        testProp(o, "keyCode");
        testConsts(KeyboardAutomationAction, "KEY_DOWN", "KEY_UP");
        trace("");

        trace("=== Testing MouseAutomationAction");
        try {
            o = new MouseAutomationAction();
        } catch(e) { trace(e); }
        o = new MouseAutomationAction("hello");
        trace("MouseAutomationAction is AutomationAction:", o is AutomationAction);
        trace("== with default values");
        testProp(o, "type");
        testProp(o, "stageX");
        testProp(o, "stageY");
        testProp(o, "delta");
        trace("== with provided values");
        o = new MouseAutomationAction("hello", 4.56, 7.89, 9.87);
        testProp(o, "type");
        testProp(o, "stageX");
        testProp(o, "stageY");
        testProp(o, "delta");
        testConsts(
            MouseAutomationAction,
            "MIDDLE_MOUSE_DOWN", "MIDDLE_MOUSE_UP",
            "MOUSE_DOWN", "MOUSE_MOVE", "MOUSE_DOWN", "MOUSE_WHEEL",
            "RIGHT_MOUSE_DOWN", "RIGHT_MOUSE_UP"
        );
        trace("");

        trace("=== Testing StageCapture");
        trace("StageCapture is EventDispatcher:", (new StageCapture()) is EventDispatcher);
        testConsts(
            StageCapture,
            "CURRENT", "MULTIPLE", "NEXT",
            "RASTER", "SCREEN", "STAGE"
        );
        trace("");

        trace("=== Testing StageCaptureEvent");
        try {
            o = new StageCaptureEvent();
        } catch(e) { trace(e); }
        o = new StageCaptureEvent("hello");
        trace("StageCaptureEvent is Event:", o is Event);
        trace("== with default values");
        testProp(o, "type");
        testProp(o, "bubbles");
        testProp(o, "cancelable");
        testProp(o, "url");
        testProp(o, "checksum");
        testProp(o, "pts");
        trace("== with provided values");
        o = new StageCaptureEvent("hello", false, true, 1.23, -4.56, 7.89);
        testProp(o, "type");
        testProp(o, "bubbles");
        testProp(o, "cancelable");
        testProp(o, "url");
        testProp(o, "checksum");
        testProp(o, "pts");
        trace("== testing toString()");
        trace(o.toString());
        trace("== testing clone");
        o = o.clone();
        testProp(o, "type");
        testProp(o, "bubbles");
        testProp(o, "cancelable");
        testProp(o, "url");
        testProp(o, "checksum");
        testProp(o, "pts");
        testConsts(StageCaptureEvent, "CAPTURE");
    }

    private function testProp(cls:*, prop:String) {
        trace(prop + ": " + cls[prop], typeof cls[prop]);

        try {
            cls[prop] = 123.45;
            trace("Successfully set " + prop + " to " + cls[prop] + ", type " + typeof cls[prop]);
            cls[prop] = -500;
            trace("Successfully set " + prop + " to " + cls[prop] + ", type " + typeof cls[prop]);
        } catch(e) {
            trace(e);
        }
    }

    private function testConsts(cls:*, ...props) {
        trace("== Testing constants");
        for (var i in props) {
            trace(props[i] + ": " + cls[props[i]], typeof cls[props[i]]);
        }
    }
}
    
}
