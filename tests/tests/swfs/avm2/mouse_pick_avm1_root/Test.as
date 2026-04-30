package  {
    
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLRequest;
    
    
    public class Test extends MovieClip {
        
        public function Test() {
            this.stage.addEventListener("click", function (e) {
                trace("Clicked on: " + e.target + " (" + e.target.name + ")");
            });
			
            var loader = new Loader();
            loader.name = "loader";
            loader.load(new URLRequest("avm1.swf"));
			addChild(loader);
        }
		
    }
    
}