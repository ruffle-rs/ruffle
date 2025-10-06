// Compile with:
//  mtasc -main -header 200:150:30 -version 7 Child7.as -swf child7.swf

class Child7 {
  static function main(current) {
    current.global = _global;
  }
}
