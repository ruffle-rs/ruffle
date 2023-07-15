package {
    import flash.display.MovieClip;
    import flash.net.NetStreamPlayOptions;
    public class Test extends MovieClip {
        public function Test() {
          var nspo:NetStreamPlayOptions = new NetStreamPlayOptions();
          trace(nspo.len);
          trace(nspo.offset);
          trace(nspo.oldStreamName);
          trace(nspo.start);
          trace(nspo.streamName);
          trace(nspo.transition);
        }
    }
}
