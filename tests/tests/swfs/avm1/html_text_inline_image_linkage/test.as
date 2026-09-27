var tf = _root.createTextField("tf", 100, 20, 20, 400, 180);
tf.html = true;
tf.multiline = true;

tf.htmlText = "A<img src='Clip' id='hello'>B";

trace("TEXT=" + tf.text);
trace("HELLO=" + tf.hello);
trace("NAME=" + tf.hello._name);
trace("FRAME=" + tf.hello._currentframe);
trace("TOTALFRAMES=" + tf.hello._totalframes);
trace("PARENT=" + tf.hello._parent);

tf.htmlText = "replacement";
trace("AFTER_CLEAR=" + tf.hello);

tf.htmlText = "A<img src='Clip' id='hello'>B";
trace("SECOND=" + tf.hello);

tf.hello = "custom";
trace("REASSIGNED=" + tf.hello);

tf.htmlText = "replacement";
trace("AFTER_REASSIGN_CLEAR=" + tf.hello);

stop();
