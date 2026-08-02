package  {
    
import flash.display.Sprite;
import flash.media.*;
import flash.utils.Dictionary;


public class Test extends Sprite {

    public function Test() {
        super();

        var o:*;

        trace("=== Testing AVABRParameters");
        o = new AVABRParameters("hello", 2.2, 3.3, 4.4);
        testProp(o, "policy");
        testProp(o, "startBitsPerSecond");
        testProp(o, "minBitsPerSecond");
        testProp(o, "maxBitsPerSecond");
        trace("");

        trace("=== Testing AVABRProfileInfo");
        o = new AVABRProfileInfo(1.1, 2.2, 3.3);
        testProp(o, "bitsPerSecond");
        testProp(o, "width");
        testProp(o, "height");
        trace("");

        trace("=== Testing AVCuePoint");
        trace("== with null dictionary");
        o = new AVCuePoint(null, 2.2);
        testProp(o, "dictionary");
        trace("== with actual dictionary");
        o = new AVCuePoint(new Dictionary(), 2.2);
        testProp(o, "dictionary");
        testProp(o, "localTime");
        trace("");

        trace("=== Testing AVInsertionResult");
        o = new AVInsertionResult(1.1, 2.2, 3.3);
        trace("o is AVResult: " + (o is AVResult));
        testProp(o, "result");
        testProp(o, "periodIndex");
        testProp(o, "insertedBeforeReadHead");
        trace("");

        trace("=== Testing AVNetworkingParams");
        trace("== with default params");
        o = new AVNetworkingParams();
        testProp(o, "forceNativeNetworking");
        testProp(o, "readSetCookieHeader");
        testProp(o, "useCookieHeaderForAllRequests");
        testProp(o, "networkDownVerificationUrl");
        testProp(o, "appendRandomQueryParameter");
        trace("== with provided params");
        o = new AVNetworkingParams(true, false, true, "hello");
        testProp(o, "forceNativeNetworking");
        testProp(o, "readSetCookieHeader");
        testProp(o, "useCookieHeaderForAllRequests");
        testProp(o, "networkDownVerificationUrl");
        testProp(o, "appendRandomQueryParameter");
        trace("");

        trace("=== Testing AVPeriodInfo");
        o = new AVPeriodInfo(1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8, 9.9, 10.1);
        testProp(o, "localStartTime");
        testProp(o, "virtualStartTime");
        testProp(o, "duration");
        testProp(o, "firstCuePointIndex");
        testProp(o, "lastCuePointIndex");
        testProp(o, "firstSubscribedTagIndex");
        testProp(o, "lastSubscribedTagIndex");
        testProp(o, "userData");
        testProp(o, "supportsTrickPlay");
        testProp(o, "targetDuration");
        trace("");

        trace("=== Testing AVPlayState");
        o = new AVPlayState(1.1);
        testProp(o, "state");
        trace("");

        trace("=== Testing AVResult");
        o = new AVResult(1.2);
        testProp(o, "result");
        trace("");

        trace("=== Testing AVTagData");
        o = new AVTagData("hello", 2.2);
        testProp(o, "data");
        testProp(o, "localTime");
        trace("");
        
        trace("=== Testing AVTimeline");
        o = new AVTimeline("hello", 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, true);
        testProp(o, "type");
        testProp(o, "virtualStartTime");
        testProp(o, "virtualDuration");
        testProp(o, "firstPeriodIndex");
        testProp(o, "lastPeriodIndex");
        testProp(o, "firstSubscribedTagIndex");
        testProp(o, "lastSubscribedTagIndex");
        testProp(o, "complete");
        trace("");

        trace("=== Testing AVTrackInfo");
        o = new AVTrackInfo("hello", "world", 3.3, 4.4, 5.5, 6.6, 7.7, 8.8);
        testProp(o, "description");
        testProp(o, "language");
        testProp(o, "defaultTrack");
        testProp(o, "autoSelect");
        testProp(o, "forced");
        testProp(o, "activity");
        testProp(o, "dataTrackInfoServiceType");
        testProp(o, "pid");
    }

    private function testProp(cls:*, prop:String) {
        trace(prop + ": " + cls[prop], typeof cls[prop]);

        try {
            cls[prop] = 123.45;
            trace("Successfully set " + prop + " to " + cls[prop] + ", type " + typeof cls[prop]);
        } catch(e) {
            trace(e);
        }
    }
}
    
}
