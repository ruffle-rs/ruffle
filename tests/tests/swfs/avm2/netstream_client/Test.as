package {
    import flash.display.MovieClip;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.utils.getQualifiedClassName;

    public class Test extends MovieClip {

        public function Test() {
            super();
            var con:NetConnection = new NetConnection();
            con.connect(null);
            var stream:NetStream = new NetStream(con);
            trace(stream.client);
            trace(stream.client == stream);
            try {
                stream.client = 3;
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            try {
                stream.client = true;
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            try {
                stream.client = "abcd";
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            try {
                stream.client = null;
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            try {
                stream.client = undefined;
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            try {
                stream.client = {};
                trace("success!");
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            try {
                stream.client = MovieClip;
                trace("success!");
            } catch (e:Error) {
                trace(getQualifiedClassName(e) + ": " + e.errorID);
            }
            trace("test over");
        }
    }
}
