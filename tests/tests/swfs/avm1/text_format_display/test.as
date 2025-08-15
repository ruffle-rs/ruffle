var tf = new TextFormat();

trace("Default");
trace(tf.display);

trace("Various values");
tf.display = '';
trace(tf.display);
tf.display = 'x';
trace(tf.display);
tf.display = new Object();
trace(tf.display);
tf.display = 1;
trace(tf.display);
tf.display = 0;
trace(tf.display);
tf.display = null;
trace(tf.display);
tf.display = undefined;
trace(tf.display);

trace("Proper values");
tf.display = 'inline';
trace(tf.display);
tf.display = 'block';
trace(tf.display);
tf.display = 'none';
trace(tf.display);
tf.display = 'Inline';
trace(tf.display);

trace("Resetting to null");
tf.display = 'inline';
trace(tf.display);
tf.display = null;
trace(tf.display);

trace("Resetting to undefined");
tf.display = 'inline';
trace(tf.display);
tf.display = undefined;
trace(tf.display);
