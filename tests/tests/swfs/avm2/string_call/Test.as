package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var cls = String;
trace('cls(): ' + cls());

trace('String(undefined): ' + String(undefined))
trace('String(null): ' + String(null))
trace('String(42): ' + String(42))
trace('String(false): ' + String(false))
trace('String("abc"): ' + String("abc"))
trace('String({}): ' + String({}))

trace('String(undefined).split(""): ' + String(undefined).split(""));
trace('String(null).split(""): ' + String(null).split(""));
trace('String(42).split(""): ' + String(42).split(""));
trace('String(false).split(""): ' + String(false).split(""));
trace('String("abc").split(""): ' + String("abc").split(""));
trace('String({}).split(""): ' + String({}).split(""));