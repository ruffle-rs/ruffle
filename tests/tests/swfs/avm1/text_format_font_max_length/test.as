function test() {
    var tf = new TextFormat("a very , very, very, very, very, very, very, very, very, very, very, very, very long font");
    trace("Constructor: " + tf.font);
    tf.font = "a 2 very , very, very, very, very, very, very, very, very, very, very, very, very long font";
    trace("Setter: " + tf.font);
}

test();
