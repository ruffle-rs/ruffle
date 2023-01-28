// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(cur) {
        var text = cur.createTextField("textField", 1, 10, 10, 150, 30);
        cur.textField.html = true;

        cur.textField.htmlText = "Misafirler kamera açamazlar lütfen üye olunuz.";
        trace(cur.textField.text);
        cur.textField.htmlText += "Misafirler kamera açamazlar lütfen üye olunuz.";
        trace(cur.textField.text);
    }
}
