// Tests TextField.StyleSheet.transform and setStyle with a movie clip that
// has `color` and `fontWeight` properties as the style object.

createEmptyMovieClip("style", getNextHighestDepth());
style.color = "#FF0000";
style.fontWeight = "bold";

var styleSheet = new TextField.StyleSheet();

var textFormat = styleSheet.transform(style);
trace("transform(clip): " + textFormat + ", color: " + textFormat.color + ", bold: " + textFormat.bold);

styleSheet.setStyle("p", style);
var storedStyle = styleSheet.getStyle("p");
trace("setStyle(\"p\", clip): getStyle: " + storedStyle + ", typeof: " + typeof(storedStyle));
trace("getStyleNames: " + styleSheet.getStyleNames());
