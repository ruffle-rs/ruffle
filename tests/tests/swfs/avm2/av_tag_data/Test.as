package {
    import flash.display.MovieClip;
    import flash.media.AVTagData;
    public class Test extends MovieClip {
        public function Test() {
          var avTag:AVTagData = new AVTagData("test string", 1);
          trace(avTag.data);
          trace(avTag.localTime);
        }
    }
}
