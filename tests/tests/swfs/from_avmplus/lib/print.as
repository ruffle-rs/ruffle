package {
/*
In avmplus, this is defined here: https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/test/util/flashrunner/template.as
They run tests via a socket, so print gets copied through that & on screen.
We don't need any of that, so just redirect it to trace :)
 */
public function print(...strings):void {
  for (var i=0; i < strings.length; i++) {
    trace(strings[i]);
  }
}
}