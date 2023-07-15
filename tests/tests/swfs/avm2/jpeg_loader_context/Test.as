package {
    import flash.display.MovieClip;
    import flash.system.JPEGLoaderContext;
    public class Test extends MovieClip {
        public function Test() {
          var jpegLoaderContext1:JPEGLoaderContext = new JPEGLoaderContext();
          trace(jpegLoaderContext1.checkPolicyFile);
          trace(jpegLoaderContext1.deblockingFilter);
          var jpegLoaderContext2:JPEGLoaderContext = new JPEGLoaderContext(0, false);
          trace(jpegLoaderContext1.checkPolicyFile);
          trace(jpegLoaderContext1.deblockingFilter);
          var jpegLoaderContext3:JPEGLoaderContext = new JPEGLoaderContext(1, true);
          trace(jpegLoaderContext3.checkPolicyFile);
          trace(jpegLoaderContext3.deblockingFilter);
        }
    }
}
