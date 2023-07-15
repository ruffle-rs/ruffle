package {
    import flash.display.MovieClip;
    import flash.media.AVNetworkingParams;
    public class Test extends MovieClip {
        public function Test() {
            var avnp1:AVNetworkingParams = new AVNetworkingParams();
            trace(avnp1.forceNativeNetworking);
            trace(avnp1.readSetCookieHeader);
            trace(avnp1.useCookieHeaderForAllRequests);
            var avnp2:AVNetworkingParams = new AVNetworkingParams(false, true, false);
            trace(avnp2.forceNativeNetworking);
            trace(avnp2.readSetCookieHeader);
            trace(avnp2.useCookieHeaderForAllRequests);
            var avnp3:AVNetworkingParams = new AVNetworkingParams(true, false, true);
            trace(avnp3.forceNativeNetworking);
            trace(avnp3.readSetCookieHeader);
            trace(avnp3.useCookieHeaderForAllRequests);
        }
    }
}
