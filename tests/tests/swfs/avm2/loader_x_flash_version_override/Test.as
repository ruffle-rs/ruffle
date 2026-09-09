package {
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.events.Event;
    import flash.events.SecurityErrorEvent;
    import flash.net.URLRequest;
    import flash.net.URLRequestHeader;

    public class Test extends MovieClip {
        public function Test() {
            this.addEventListener(Event.ENTER_FRAME, onFrame);
        }

        function onFrame(event:Event):void {
            this.removeEventListener(Event.ENTER_FRAME, onFrame);

            // A user-supplied `X-Flash-Version` URLRequestHeader is meant to
            // override the default Ruffle inserts in `request_from_url_request`
            // (#15276).  The default is inserted first, so the user value wins
            // via IndexMap's last-write-wins on insert while staying at the
            // original (first) position.  The surrounding `MyBefore` and
            // `MyAfter` headers prove the override doesn't disturb the rest of
            // the user-headers loop.
            var request:URLRequest = new URLRequest();
            request.url = "http://localhost:8000";
            request.method = "GET";

            var headers:Array = new Array();
            headers.push(new URLRequestHeader("MyBefore", "before-val"));
            headers.push(new URLRequestHeader("X-Flash-Version", "99,0,1,2"));
            headers.push(new URLRequestHeader("MyAfter", "after-val"));
            request.requestHeaders = headers;

            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(e:SecurityErrorEvent):void {
                trace("Security error: " + e);
            });
            loader.load(request);
        }
    }
}
