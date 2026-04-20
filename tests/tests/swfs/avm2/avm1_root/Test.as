package  {
    
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.net.URLRequest;

    
    public class Test extends MovieClip {
        
        
        public function Test() {
            super();
            var loader:Loader = new Loader();
            loader.load(new URLRequest("avm1_child.swf"));
            addChild(loader);
        }
    }
    
}