package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var settings = XML.settings();
trace("XML.settings(): " + settings);

trace("settings.ignoreComments = " + settings.ignoreComments);
trace("settings.ignoreWhitespace = " + settings.ignoreWhitespace);
trace("settings.ignoreProcessingInstructions = " + settings.ignoreProcessingInstructions);
trace("settings.prettyIndent = " + settings.prettyIndent);
trace("settings.prettyPrinting = " + settings.prettyPrinting);

// Stub
XML.setSettings(settings);
