package 
{ 
    import flash.display.BlendMode; 
    import flash.display.GradientType; 
    import flash.display.Graphics; 
    import flash.display.Shader; 
    import flash.display.Shape; 
    import flash.display.Sprite; 
    import flash.events.Event; 
    import flash.geom.Matrix; 
    import flash.net.URLLoader; 
    import flash.net.URLLoaderDataFormat; 
    import flash.net.URLRequest; 
    
	// Based on https://help.adobe.com/en_US/as3/dev/WSB19E965E-CCD2-4174-8077-8E5D0141A4A8.html
    public class Test extends Sprite 
    { 
        private var shader:Shader; 
        private var loader:URLLoader; 
         
        public function Test() 
        {
            init(); 
        } 
         
        private function init():void 
        { 
            loader = new URLLoader(); 
            loader.dataFormat = URLLoaderDataFormat.BINARY; 
            loader.addEventListener(Event.COMPLETE, onLoadComplete); 
			loader.addEventListener("ioError", function(e) {
				trace("IO error: " + e);
			})
            loader.load(new URLRequest("LumaLighten.pbj")); 
        } 
         
         
        private function onLoadComplete(event:Event):void 
        { 
			trace("Loaded!");
            shader = new Shader(loader.data); 
             
            var backdrop:Shape = new Shape(); 
            var g0:Graphics = backdrop.graphics; 
            g0.beginFill(0x303030); 
            g0.drawRect(0, 0, 400, 200); 
            g0.endFill(); 
            addChild(backdrop); 
             
            var backgroundShape:Shape = new Shape(); 
            var g1:Graphics = backgroundShape.graphics; 
            var c1:Array = [0x336600, 0x80ff00]; 
            var a1:Array = [255, 255]; 
            var r1:Array = [100, 255]; 
            var m1:Matrix = new Matrix(); 
            m1.createGradientBox(300, 200); 
            g1.beginGradientFill(GradientType.LINEAR, c1, a1, r1, m1); 
            g1.drawEllipse(0, 0, 300, 200); 
            g1.endFill(); 
            addChild(backgroundShape); 
             
            var foregroundShape:Shape = new Shape(); 
            var g2:Graphics = foregroundShape.graphics; 
            var c2:Array = [0xff8000, 0x663300]; 
            var a2:Array = [255, 255]; 
            var r2:Array = [100, 255]; 
            var m2:Matrix = new Matrix(); 
            m2.createGradientBox(300, 200); 
            g2.beginGradientFill(GradientType.LINEAR, c2, a2, r2, m2); 
            g2.drawEllipse(100, 0, 300, 200); 
            g2.endFill(); 
            addChild(foregroundShape);
			
			try {
				foregroundShape.blendShader = new Shader();
			} catch (e) {
				trace("Caught error: " + e);
			}
			
            foregroundShape.blendShader = shader;
			trace("After blendShader: " + foregroundShape.blendMode);
			// Ensure that blendShader is persisted across a blendMode change
			// (we can't access it directly, but the rendered result will use the shader)
			foregroundShape.blendMode = BlendMode.NORMAL;
			foregroundShape.blendMode = BlendMode.SHADER;
        } 
    } 
}