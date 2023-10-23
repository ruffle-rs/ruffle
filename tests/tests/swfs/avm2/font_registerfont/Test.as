package  {
	
	import flash.display.MovieClip;
	import flash.text.Font;
	import flash.display.Loader;
	import flash.system.LoaderContext;
	import flash.system.ApplicationDomain;
	import flash.events.Event;
	import flash.net.URLRequest;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			dumpFonts();
			
			var loader: Loader = new Loader();
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onFontLoaded);
			loader.load(new URLRequest("font.swf"));
		}
		
		function onFontLoaded(event: Event) {
			trace("[test.swf] // onFontLoaded");
			dumpFonts();
			
			var loader: Loader = new Loader();
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onFontLoadedAgain);
			loader.load(new URLRequest("font.swf"));
		}
		
		function onFontLoadedAgain(event: Event) {
			trace("[test.swf] // onFontLoaded a second time");
			dumpFonts();
		}
		
		function dumpFonts() {
			trace("[test.swf] // var fonts = enumerateFonts(false)");
			var fonts = Font.enumerateFonts(false);
			
			trace("[test.swf] // fonts.length");
			trace("[test.swf] "+ fonts.length);
			trace("[test.swf]");
			
			for (var i = 0; i < fonts.length; i++) {
				dumpFont("[test.swf] fonts[" + i + "]", fonts[i]);
			}
			trace("[test.swf]");
		}
		
		function dumpFont(name: String, font: Font) {
			trace("[test.swf] // " + name + ".fontName");
			trace("[test.swf] " + font.fontName);
			trace("[test.swf]");
			
			trace("[test.swf] // " + name + ".fontStyle");
			trace("[test.swf] " + font.fontStyle);
			trace("[test.swf]");
			
			trace("[test.swf] // " + name + ".fontType");
			trace("[test.swf] " + font.fontType);
			trace("[test.swf]");
			
			trace("[test.swf]");
		}
	}
	
}
