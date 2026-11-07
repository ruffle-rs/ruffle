package {
    import flash.display.MovieClip;
    import flash.net.navigateToURL;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        public function Test() {
            try {
                navigateToURL(new URLRequest("https://example.com/"), "_blank");
            } catch (error:*) {
                trace(error.constructor);
                trace(error.getStackTrace());
            }
        }
    }
}
