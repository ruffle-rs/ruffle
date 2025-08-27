package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {

        public function Test() {
            super();
            var v:Vector.<int> = new Vector.<int>();
            v.push(1);
            var a:Array = [1];
            trace(a.every(null));
            trace(a.filter(null));
            trace(a.forEach(null));
            trace(a.map(null));
            trace(a.some(null));
            trace(v.every(null));
            trace(v.filter(null));
            trace(v.forEach(null));
            trace(v.map(null));
            trace(v.some(null));
        }
    }
}
