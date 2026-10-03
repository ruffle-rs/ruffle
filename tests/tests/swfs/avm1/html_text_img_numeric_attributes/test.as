var tf = _root.createTextField("tf", 1, 0, 0, 400, 300);

tf.html = true;
tf.multiline = true;
tf.condenseWhite = false;

var format = new TextFormat();
format.font = "Times";
format.size = 12;
tf.setNewTextFormat(format);

var html = "A<img src='missing.jpg' width='40.9px' height='30.7xyz' hspace='4.9px' vspace='6.2xyz'>B";

tf.htmlText = html;

trace("SOURCE=" + html);
trace("TEXT=" + tf.text);
trace("HTML=" + tf.htmlText);

stop();
