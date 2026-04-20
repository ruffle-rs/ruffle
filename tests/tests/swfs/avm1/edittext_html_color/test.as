var text = _root.createTextField("text", 10, 0, 0, 10, 10);
text.html = true;

var tf = new TextFormat();
tf.color = 0x1;
text.setNewTextFormat(tf);

function checkColor(color) {
    var html = "<font color=\"#123456\">x<font color=\"" + color + "\">inner</font>x</font>";
    text.htmlText = html;

    trace("Color: " + color + ";");
    trace("  default color: " + text.getNewTextFormat().color.toString(16));
    trace("  inner color: " + text.getTextFormat(2, 5).color.toString(16));
}

checkColor("202020");
checkColor("#202020");
checkColor("#2");
checkColor("#23");
checkColor("#234");
checkColor("#2345");
checkColor("#23456");
checkColor("#234567");
checkColor("#2345678");
checkColor("#23456789");
checkColor("#23456789a");
checkColor("#abf");
checkColor("#abfg");
checkColor("#g101010");
checkColor("#g234567");
checkColor("#g2345678");
checkColor("#g23456789abc");
checkColor("#g23456789abcd");
checkColor("#2g");
checkColor("#23g");
checkColor("#234g");
checkColor("#2345g");
checkColor("#23456789ag");
checkColor("#123456789abcdef123456789abcdef123456789abcdef123456789abcdef");

checkColor(" #234567");
checkColor("# 234567");
checkColor("#2 34567");
checkColor("#234567 ");
checkColor("#     234567");
checkColor("#\t234567");
checkColor("#   234567  ");

checkColor("#abc");
checkColor("#ABC");
checkColor("#AbC");

checkColor("#0123456789abcdefABCDEF2a4");

checkColor("#g");
checkColor("# ");

checkColor("#23g45");
